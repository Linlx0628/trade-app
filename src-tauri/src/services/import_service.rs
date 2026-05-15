use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub total_rows: u64,
    pub imported: u64,
    pub skipped: u64,
    pub errors: Vec<String>,
}

pub struct ImportService;

impl ImportService {
    pub fn preview_csv(file_path: &str) -> Result<ImportPreview, AppError> {
        let is_xlsx = file_path.to_lowercase().ends_with(".xlsx");
        if is_xlsx {
            Self::preview_xlsx(file_path)
        } else {
            Self::preview_csv_file(file_path)
        }
    }

    fn preview_csv_file(file_path: &str) -> Result<ImportPreview, AppError> {
        let mut content = String::new();
        File::open(file_path)
            .map_err(|e| AppError::Database(format!("无法打开文件: {}", e)))?
            .read_to_string(&mut content)
            .map_err(|e| AppError::Database(format!("无法读取文件: {}", e)))?;

        let content = content.trim_start_matches('\u{FEFF}');
        let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(content.as_bytes());

        let headers = reader.headers()
            .map_err(|e| AppError::Database(format!("无法解析CSV头: {}", e)))?
            .iter().map(String::from).collect();

        let mut rows: Vec<Vec<String>> = Vec::new();
        for record in reader.records() {
            if rows.len() >= 10 { break; }
            if let Ok(r) = record { rows.push(r.iter().map(String::from).collect()); }
        }

        let total_lines = content.lines().count().saturating_sub(1);
        Ok(ImportPreview { headers, rows, total_lines })
    }

    fn preview_xlsx(file_path: &str) -> Result<ImportPreview, AppError> {
        use calamine::{open_workbook_auto, Reader};

        let mut wb = open_workbook_auto(file_path)
            .map_err(|e| AppError::Database(format!("无法打开Excel文件: {}", e)))?;

        let sheet_names = wb.sheet_names().to_vec();
        let sheet_name = sheet_names.first()
            .ok_or_else(|| AppError::Database("Excel文件没有工作表".to_string()))?
            .clone();

        let range = wb.worksheet_range(&sheet_name)
            .map_err(|e| AppError::Database(format!("读取工作表失败: {}", e)))?;

        let mut rows_iter = range.rows();
        let headers: Vec<String> = rows_iter.next()
            .map(|r| r.iter().map(|c| c.to_string()).collect())
            .unwrap_or_default();

        let mut preview_rows: Vec<Vec<String>> = Vec::new();
        let mut total_lines = 0;
        for row in rows_iter {
            total_lines += 1;
            if preview_rows.len() < 10 {
                preview_rows.push(row.iter().map(|c| c.to_string()).collect());
            }
        }

        Ok(ImportPreview { headers, rows: preview_rows, total_lines })
    }

    pub fn import_trade_logs(
        state: &State<'_, DbState>,
        file_path: &str,
        account_id: &str,
    ) -> Result<ImportResult, AppError> {
        let is_xlsx = file_path.to_lowercase().ends_with(".xlsx");
        if is_xlsx {
            Self::import_from_xlsx(state, file_path, account_id)
        } else {
            Self::import_from_csv(state, file_path, account_id)
        }
    }

    fn import_from_csv(
        state: &State<'_, DbState>,
        file_path: &str,
        account_id: &str,
    ) -> Result<ImportResult, AppError> {
        let mut content = String::new();
        File::open(file_path)
            .map_err(|e| AppError::Database(format!("无法打开文件: {}", e)))?
            .read_to_string(&mut content)
            .map_err(|e| AppError::Database(format!("无法读取文件: {}", e)))?;

        let content = content.trim_start_matches('\u{FEFF}');
        let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(content.as_bytes());

        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        let mut result = ImportResult { total_rows: 0, imported: 0, skipped: 0, errors: Vec::new() };

        for record in reader.records() {
            result.total_rows += 1;
            match record {
                Ok(r) => {
                    if let Err(e) = insert_log_row(&conn, account_id, r.iter().map(String::from).collect(), &mut result) {
                        result.errors.push(format!("第 {} 行: {}", result.total_rows, e));
                    }
                }
                Err(e) => { result.skipped += 1; result.errors.push(format!("第 {} 行: 解析失败 {}", result.total_rows, e)); }
            }
        }
        Ok(result)
    }

    fn import_from_xlsx(
        state: &State<'_, DbState>,
        file_path: &str,
        account_id: &str,
    ) -> Result<ImportResult, AppError> {
        use calamine::{open_workbook_auto, Reader};

        let mut wb = open_workbook_auto(file_path)
            .map_err(|e| AppError::Database(format!("无法打开Excel文件: {}", e)))?;

        let sheet_names = wb.sheet_names().to_vec();
        let sheet_name = sheet_names.first()
            .ok_or_else(|| AppError::Database("Excel文件没有工作表".to_string()))?
            .clone();

        let range = wb.worksheet_range(&sheet_name)
            .map_err(|e| AppError::Database(format!("读取工作表失败: {}", e)))?;

        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        let mut result = ImportResult { total_rows: 0, imported: 0, skipped: 0, errors: Vec::new() };

        let mut first = true;
        for row in range.rows() {
            if first { first = false; continue; } // skip header
            result.total_rows += 1;
            let cells: Vec<String> = row.iter().map(|c| c.to_string()).collect();
            if let Err(e) = insert_log_row(&conn, account_id, cells, &mut result) {
                result.errors.push(format!("第 {} 行: {}", result.total_rows, e));
            }
        }
        Ok(result)
    }
}

fn insert_log_row(
    conn: &rusqlite::Connection,
    account_id: &str,
    cells: Vec<String>,
    result: &mut ImportResult,
) -> Result<(), String> {
    if cells.len() < 12 {
        result.skipped += 1;
        return Err("字段不足".to_string());
    }

    let get = |i: usize| cells.get(i).map(|s| s.trim().to_string()).unwrap_or_default();

    let symbol = get(0);
    if symbol.is_empty() { result.skipped += 1; return Ok(()); }

    let direction_raw = get(2);
    let direction = if direction_raw == "做多" || direction_raw == "long" { "long" } else { "short" };
    let entry_price: f64 = get(4).parse().unwrap_or(0.0);
    let exit_price: f64 = get(5).parse().unwrap_or(0.0);
    let stop_loss: f64 = get(6).parse().unwrap_or(0.0);
    let lots: f64 = get(7).parse().unwrap_or(0.0);
    let pnl: f64 = get(8).parse().unwrap_or(0.0);
    let status_raw = get(11);
    let status = if status_raw == "已平仓" || status_raw == "closed" { "closed" } else { "open" };

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO trade_log (id, account_id, plan_id, symbol, name, direction, entry_price, exit_price, stop_loss, lots, market_type, status, entry_time, exit_time, pnl, pnl_points, commission, tags, notes, emotion_before, emotion_after, confidence, created_at, updated_at)
         VALUES (?1, ?2, '', ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'futures', ?10, ?11, ?12, ?13, 0, 0, '[]', '', '', '', 5, ?14, ?15)",
        rusqlite::params![
            id, account_id, symbol, get(1), direction, entry_price, exit_price, stop_loss, lots, status,
            get(12), get(13), pnl, now, now
        ]
    ).map_err(|e| format!("{}", e))?;

    result.imported += 1;
    Ok(())
}
