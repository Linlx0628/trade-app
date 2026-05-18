use crate::db::DbState;
use crate::error::AppError;
use crate::models::backup::{BackupConfig, BackupInfo, BackupStatus};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

static LAST_BACKUP: Mutex<Option<String>> = Mutex::new(None);

pub struct AutoBackupService;

impl AutoBackupService {
    fn resolve_backup_dir(config: &BackupConfig, app_data_dir: &Path) -> PathBuf {
        if config.backup_dir.is_empty() {
            app_data_dir.join("backups")
        } else {
            PathBuf::from(&config.backup_dir)
        }
    }

    pub fn perform_backup(
        state: &State<'_, DbState>,
        config: &BackupConfig,
        app_data_dir: &Path,
    ) -> Result<BackupInfo, AppError> {
        let backup_dir = Self::resolve_backup_dir(config, app_data_dir);
        fs::create_dir_all(&backup_dir)
            .map_err(|e| AppError::Database(format!("无法创建备份目录: {}", e)))?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("trademind_backup_{}.db", timestamp);
        let backup_path = backup_dir.join(&filename);

        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;

        let backup_path_str = backup_path.to_string_lossy().to_string();
        conn.execute(&format!("VACUUM INTO '{}'", backup_path_str), [])
            .map_err(|e| AppError::Database(format!("备份失败: {}", e)))?;

        drop(conn);

        let metadata = fs::metadata(&backup_path)
            .map_err(|e| AppError::Database(format!("无法读取备份文件信息: {}", e)))?;

        let info = BackupInfo {
            filename,
            file_size: metadata.len(),
            created_at: chrono::Local::now().to_rfc3339(),
        };

        if let Ok(mut last) = LAST_BACKUP.lock() {
            *last = Some(info.created_at.clone());
        }

        if config.max_backups > 0 {
            let _ = Self::cleanup_old_backups(&backup_dir, config.max_backups);
        }

        Ok(info)
    }

    fn cleanup_old_backups(backup_dir: &Path, max_count: u32) -> Result<(), AppError> {
        let mut entries: Vec<(String, String)> = Vec::new();
        let read_dir = fs::read_dir(backup_dir)
            .map_err(|e| AppError::Database(format!("无法读取备份目录: {}", e)))?;

        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("trademind_backup_") && name.ends_with(".db") {
                entries.push((name, entry.path().to_string_lossy().to_string()));
            }
        }

        entries.sort_by(|a, b| b.0.cmp(&a.0));

        if (entries.len() as u32) > max_count {
            for (_, path) in entries.iter().skip(max_count as usize) {
                let _ = fs::remove_file(path);
                let json_path = path.replace(".db", ".json");
                let _ = fs::remove_file(&json_path);
            }
        }

        Ok(())
    }

    pub fn list_backups(backup_dir: &Path) -> Result<Vec<BackupInfo>, AppError> {
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups: Vec<BackupInfo> = Vec::new();
        let read_dir = fs::read_dir(backup_dir)
            .map_err(|e| AppError::Database(format!("无法读取备份目录: {}", e)))?;

        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("trademind_backup_") && name.ends_with(".db") {
                if let Ok(metadata) = entry.metadata() {
                    let created = metadata.created().ok().and_then(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).ok()
                    }).map(|d| {
                        chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_default()
                    }).unwrap_or_default();

                    backups.push(BackupInfo {
                        filename: name,
                        file_size: metadata.len(),
                        created_at: created,
                    });
                }
            }
        }

        backups.sort_by(|a, b| b.filename.cmp(&a.filename));
        Ok(backups)
    }

    pub fn restore_from_backup(
        state: &State<'_, DbState>,
        app_data_dir: &Path,
        filename: &str,
        backup_dir: &Path,
    ) -> Result<(), AppError> {
        let backup_file = backup_dir.join(filename);
        if !backup_file.exists() {
            return Err(AppError::NotFound(format!("备份文件不存在: {}", filename)));
        }

        let db_path = app_data_dir.join("trademind.db");

        {
            let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
                .map_err(|e| AppError::Database(format!("检查点失败: {}", e)))?;
        }

        let temp_path = app_data_dir.join("trademind_restore_temp.db");
        fs::copy(&backup_file, &temp_path)
            .map_err(|e| AppError::Database(format!("复制备份文件失败: {}", e)))?;

        let temp_conn = Connection::open(&temp_path)
            .map_err(|e| AppError::Database(format!("无法打开临时数据库: {}", e)))?;

        let db_path_str = db_path.to_string_lossy().to_string();
        temp_conn.execute(&format!("VACUUM INTO '{}'", db_path_str), [])
            .map_err(|e| AppError::Database(format!("恢复数据失败: {}", e)))?;

        drop(temp_conn);
        let _ = fs::remove_file(&temp_path);

        Ok(())
    }

    pub fn get_status(backup_dir: &Path, config: &BackupConfig) -> BackupStatus {
        let backups = Self::list_backups(backup_dir).unwrap_or_default();
        let total_size: u64 = backups.iter().map(|b| b.file_size).sum();

        let last_backup_at = LAST_BACKUP.lock().ok().and_then(|last| last.clone());

        let next_backup_at = if config.enabled && config.interval_minutes > 0 {
            last_backup_at.as_ref().and_then(|last| {
                chrono::DateTime::parse_from_rfc3339(last).ok().map(|dt| {
                    (dt + chrono::Duration::minutes(config.interval_minutes as i64))
                        .format("%Y-%m-%d %H:%M:%S").to_string()
                })
            })
        } else {
            None
        };

        BackupStatus {
            last_backup_at,
            next_backup_at,
            total_backups: backups.len() as u32,
            backup_dir: backup_dir.to_string_lossy().to_string(),
            total_size_mb: total_size as f64 / (1024.0 * 1024.0),
            enabled: config.enabled,
        }
    }
}
