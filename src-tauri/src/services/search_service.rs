use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub subtitle: String,
    pub symbol: Option<String>,
    pub date: String,
    pub match_field: String,
}

pub struct SearchService;

impl SearchService {
    pub fn search(state: &State<'_, DbState>, account_id: &str, query: &str, limit: i64) -> Result<Vec<SearchResult>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        let pattern = format!("%{}%", query);
        let mut results = Vec::new();

        // Search trade plans
        let mut stmt = conn.prepare(
            "SELECT id, symbol, name, direction, status, created_at FROM trade_plan WHERE account_id = ?1 AND (symbol LIKE ?2 OR name LIKE ?2 OR strategy LIKE ?2 OR notes LIKE ?2) ORDER BY created_at DESC LIMIT ?3"
        ).map_err(|e| AppError::Database(format!("{}", e)))?;

        let plan_rows = stmt.query_map(rusqlite::params![account_id, pattern, limit], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                item_type: "plan".to_string(),
                title: format!("{} {}", row.get::<_, String>(1)?, row.get::<_, String>(2).unwrap_or_default()).trim().to_string(),
                subtitle: format!("交易计划 · {} · {}", row.get::<_, String>(3)?, row.get::<_, String>(4)?),
                symbol: Some(row.get(5)?),
                date: row.get::<_, String>(5)?,
                match_field: "plan".to_string(),
            })
        }).map_err(|e| AppError::Database(format!("{}", e)))?;

        for r in plan_rows.filter_map(|r| r.ok()) { results.push(r); }

        // Search trade logs
        let mut stmt = conn.prepare(
            "SELECT id, symbol, name, direction, status, entry_time FROM trade_log WHERE account_id = ?1 AND (symbol LIKE ?2 OR name LIKE ?2 OR notes LIKE ?2) ORDER BY entry_time DESC LIMIT ?3"
        ).map_err(|e| AppError::Database(format!("{}", e)))?;

        let log_rows = stmt.query_map(rusqlite::params![account_id, pattern, limit], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                item_type: "log".to_string(),
                title: format!("{} {}", row.get::<_, String>(1)?, row.get::<_, String>(2).unwrap_or_default()).trim().to_string(),
                subtitle: format!("交易日志 · {} · {}", row.get::<_, String>(3)?, row.get::<_, String>(4)?),
                symbol: Some(row.get(1)?),
                date: row.get::<_, String>(5)?,
                match_field: "log".to_string(),
            })
        }).map_err(|e| AppError::Database(format!("{}", e)))?;

        for r in log_rows.filter_map(|r| r.ok()) { results.push(r); }

        // Search summaries
        let mut stmt = conn.prepare(
            "SELECT id, summary_type, summary_date, market_view FROM trade_summary WHERE account_id = ?1 AND (market_view LIKE ?2 OR lessons LIKE ?2 OR improvement LIKE ?2) ORDER BY summary_date DESC LIMIT ?3"
        ).map_err(|e| AppError::Database(format!("{}", e)))?;

        let summary_rows = stmt.query_map(rusqlite::params![account_id, pattern, limit], |row| {
            let st: String = row.get(1)?;
            let type_label = if st == "daily" { "日总结" } else if st == "weekly" { "周总结" } else { "月总结" };
            Ok(SearchResult {
                id: row.get(0)?,
                item_type: "summary".to_string(),
                title: format!("{} - {}", row.get::<_, String>(2)?, type_label),
                subtitle: format!("交易总结 · {}", row.get::<_, String>(3).unwrap_or_default().chars().take(30).collect::<String>()),
                symbol: None,
                date: row.get::<_, String>(2)?,
                match_field: "summary".to_string(),
            })
        }).map_err(|e| AppError::Database(format!("{}", e)))?;

        for r in summary_rows.filter_map(|r| r.ok()) { results.push(r); }

        results.sort_by(|a, b| b.date.cmp(&a.date));
        results.truncate(limit as usize);

        Ok(results)
    }
}
