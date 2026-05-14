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
        let mut content = String::new();
        File::open(file_path)
            .map_err(|e| AppError::Database(format!("无法打开文件: {}", e)))?
            .read_to_string(&mut content)
            .map_err(|e| AppError::Database(format!("无法读取文件: {}", e)))?;

        let content = content.trim_start_matches('\u{FEFF}');
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(content.as_bytes());

        let headers = reader.headers()
            .map_err(|e| AppError::Database(format!("无法解析CSV头: {}", e)))?
            .iter()
            .map(String::from)
            .collect();

        let mut rows: Vec<Vec<String>> = Vec::new();
        for record in reader.records() {
            if rows.len() >= 10 { break; }
            if let Ok(r) = record {
                rows.push(r.iter().map(String::from).collect());
            }
        }

        let total_lines = content.lines().count().saturating_sub(1);
        Ok(ImportPreview { headers, rows, total_lines })
    }

    pub fn import_trade_logs_csv(
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
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(content.as_bytes());

        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        let mut result = ImportResult { total_rows: 0, imported: 0, skipped: 0, errors: Vec::new() };

        for record in reader.records() {
            result.total_rows += 1;
            match record {
                Ok(r) => {
                    if r.len() < 12 {
                        result.skipped += 1;
                        result.errors.push(format!("第 {} 行: 字段不足", result.total_rows));
                        continue;
                    }

                    let symbol = r.get(0).unwrap_or("").trim().to_string();
                    if symbol.is_empty() { result.skipped += 1; continue; }

                    let direction_raw = r.get(2).unwrap_or("long").trim();
                    let direction = if direction_raw == "做多" || direction_raw == "long" { "long" } else { "short" };
                    let entry_price: f64 = r.get(4).unwrap_or("0").parse().unwrap_or(0.0);
                    let exit_price: f64 = r.get(5).unwrap_or("0").parse().unwrap_or(0.0);
                    let stop_loss: f64 = r.get(6).unwrap_or("0").parse().unwrap_or(0.0);
                    let lots: f64 = r.get(7).unwrap_or("0").parse().unwrap_or(0.0);
                    let pnl: f64 = r.get(8).unwrap_or("0").parse().unwrap_or(0.0);
                    let status_raw = r.get(11).unwrap_or("open").trim();
                    let status = if status_raw == "已平仓" || status_raw == "closed" { "closed" } else { "open" };

                    let id = uuid::Uuid::new_v4().to_string();
                    let now = chrono::Utc::now().to_rfc3339();

                    match conn.execute(
                        "INSERT INTO trade_log (id, account_id, plan_id, symbol, name, direction, entry_price, exit_price, stop_loss, lots, market_type, status, entry_time, exit_time, pnl, pnl_points, commission, tags, notes, emotion_before, emotion_after, confidence, created_at, updated_at)
                         VALUES (?1, ?2, '', ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'futures', ?10, ?11, ?12, ?13, 0, 0, '[]', '', '', '', 5, ?14, ?15)",
                        rusqlite::params![
                            id, account_id, symbol,
                            r.get(1).unwrap_or("").trim().to_string(),
                            direction, entry_price, exit_price, stop_loss, lots, status,
                            r.get(12).unwrap_or("").trim().to_string(),
                            r.get(13).unwrap_or("").trim().to_string(),
                            pnl, now, now
                        ]
                    ) {
                        Ok(_) => result.imported += 1,
                        Err(e) => { result.skipped += 1; result.errors.push(format!("第 {} 行: {}", result.total_rows, e)); }
                    }
                }
                Err(e) => { result.skipped += 1; result.errors.push(format!("第 {} 行: 解析失败 {}", result.total_rows, e)); }
            }
        }
        Ok(result)
    }
}
