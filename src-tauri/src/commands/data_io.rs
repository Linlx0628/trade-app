use crate::db::DbState;
use crate::error::AppError;
use crate::services::{ExportService, ImportService};
use crate::services::import_service::ImportResult;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewResponse {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_lines: usize,
}

#[tauri::command]
pub fn export_trade_logs_csv(
    state: State<'_, DbState>,
    account_id: String,
    file_path: String,
) -> Result<String, AppError> {
    ExportService::export_trade_logs_csv(&state, &account_id, &file_path)
}

#[tauri::command]
pub fn export_trade_plans_csv(
    state: State<'_, DbState>,
    account_id: String,
    file_path: String,
) -> Result<String, AppError> {
    ExportService::export_trade_plans_csv(&state, &account_id, &file_path)
}

#[tauri::command]
pub fn preview_import_csv(
    file_path: String,
) -> Result<ImportPreviewResponse, AppError> {
    let preview = ImportService::preview_csv(&file_path)?;
    Ok(ImportPreviewResponse {
        headers: preview.headers,
        rows: preview.rows,
        total_lines: preview.total_lines,
    })
}

#[tauri::command]
pub fn import_trade_logs_csv(
    state: State<'_, DbState>,
    file_path: String,
    account_id: String,
) -> Result<ImportResult, AppError> {
    ImportService::import_trade_logs_csv(&state, &file_path, &account_id)
}

#[tauri::command]
pub fn create_backup(
    state: State<'_, DbState>,
    backup_dir: String,
) -> Result<String, AppError> {
    ExportService::create_backup(&state, &backup_dir)
}
