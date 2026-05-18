use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub enabled: bool,
    pub backup_dir: String,
    pub interval_minutes: u64,
    pub max_backups: u32,
    pub backup_on_start: bool,
    pub backup_on_close: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backup_dir: String::new(),
            interval_minutes: 120,
            max_backups: 30,
            backup_on_start: true,
            backup_on_close: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub filename: String,
    pub file_size: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub last_backup_at: Option<String>,
    pub next_backup_at: Option<String>,
    pub total_backups: u32,
    pub backup_dir: String,
    pub total_size_mb: f64,
    pub enabled: bool,
}
