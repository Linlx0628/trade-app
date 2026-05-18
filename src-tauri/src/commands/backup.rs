use crate::db::DbState;
use crate::error::AppError;
use crate::models::backup::{BackupConfig, BackupInfo, BackupStatus};
use crate::services::auto_backup_service::AutoBackupService;
use std::path::PathBuf;
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;

const CONFIG_KEY: &str = "auto_backup_config";

fn load_config(app: &tauri::AppHandle) -> BackupConfig {
    let store = match app.store("backup.json") {
        Ok(s) => s,
        Err(_) => return BackupConfig::default(),
    };
    store
        .get(CONFIG_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn save_config(app: &tauri::AppHandle, config: &BackupConfig) -> Result<(), AppError> {
    let store = app.store("backup.json")
        .map_err(|e| AppError::Database(format!("无法打开存储: {}", e)))?;
    store.set(CONFIG_KEY, serde_json::to_value(config).unwrap());
    store.save()
        .map_err(|e| AppError::Database(format!("持久化配置失败: {}", e)))?;
    Ok(())
}

fn get_app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Database(format!("无法获取应用数据目录: {}", e)))
}

fn resolve_backup_dir(config: &BackupConfig, app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let app_dir = get_app_data_dir(app)?;
    if config.backup_dir.is_empty() {
        Ok(app_dir.join("backups"))
    } else {
        Ok(PathBuf::from(&config.backup_dir))
    }
}

#[tauri::command]
pub fn get_backup_config(app: tauri::AppHandle) -> Result<BackupConfig, AppError> {
    let mut config = load_config(&app);
    if config.backup_dir.is_empty() {
        let app_dir = get_app_data_dir(&app)?;
        config.backup_dir = app_dir.join("backups").to_string_lossy().to_string();
    }
    Ok(config)
}

#[tauri::command]
pub fn update_backup_config(
    app: tauri::AppHandle,
    config: BackupConfig,
) -> Result<BackupConfig, AppError> {
    save_config(&app, &config)?;
    Ok(config)
}

#[tauri::command]
pub fn perform_backup(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
) -> Result<BackupInfo, AppError> {
    let config = load_config(&app);
    let app_dir = get_app_data_dir(&app)?;
    AutoBackupService::perform_backup(&state, &config, &app_dir)
}

#[tauri::command]
pub fn list_backups(app: tauri::AppHandle) -> Result<Vec<BackupInfo>, AppError> {
    let config = load_config(&app);
    let backup_dir = resolve_backup_dir(&config, &app)?;
    AutoBackupService::list_backups(&backup_dir)
}

#[tauri::command]
pub fn restore_backup(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    filename: String,
) -> Result<(), AppError> {
    let config = load_config(&app);
    let app_dir = get_app_data_dir(&app)?;
    let backup_dir = resolve_backup_dir(&config, &app)?;
    AutoBackupService::restore_from_backup(&state, &app_dir, &filename, &backup_dir)
}

#[tauri::command]
pub fn delete_backup(app: tauri::AppHandle, filename: String) -> Result<(), AppError> {
    let config = load_config(&app);
    let backup_dir = resolve_backup_dir(&config, &app)?;
    let path = backup_dir.join(&filename);
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| AppError::Database(format!("删除备份失败: {}", e)))?;
        let json_path = path.with_extension("json");
        let _ = std::fs::remove_file(&json_path);
    }
    Ok(())
}

#[tauri::command]
pub fn get_backup_status(app: tauri::AppHandle) -> Result<BackupStatus, AppError> {
    let config = load_config(&app);
    let backup_dir = resolve_backup_dir(&config, &app)?;
    Ok(AutoBackupService::get_status(&backup_dir, &config))
}
