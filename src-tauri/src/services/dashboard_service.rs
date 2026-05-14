use crate::db::DbState;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub balance: f64,
    pub today_pnl: f64,
    pub total_pnl: f64,
    pub total_trades: i64,
    pub win_trades: i64,
    pub loss_trades: i64,
    pub win_rate: f64,
    pub month_trades: i64,
    pub month_pnl: f64,
    pub open_positions: i64,
    pub pending_plans: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlTrend {
    pub date: String,
    pub pnl: f64,
    pub cumulative_pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPnl {
    pub symbol: String,
    pub name: String,
    pub trade_count: i64,
    pub total_pnl: f64,
    pub win_rate: f64,
}

pub struct DashboardService;

impl DashboardService {
    pub fn get_stats(state: &State<'_, DbState>, account_id: &str) -> Result<DashboardStats, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;

        let account: Option<(f64,)> = conn.query_row(
            "SELECT balance FROM account WHERE id = ?1", [account_id], |r| Ok((r.get(0)?,))
        ).ok();

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let month_start = chrono::Local::now().format("%Y-%m-01").to_string();

        let total_trades: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_log WHERE account_id = ?1 AND status = 'closed'",
            [account_id], |r| r.get(0)
        ).unwrap_or(0);

        let win_trades: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND pnl > 0",
            [account_id], |r| r.get(0)
        ).unwrap_or(0);

        let loss_trades: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND pnl <= 0",
            [account_id], |r| r.get(0)
        ).unwrap_or(0);

        let total_pnl: f64 = conn.query_row(
            "SELECT COALESCE(SUM(pnl), 0) FROM trade_log WHERE account_id = ?1 AND status = 'closed'",
            [account_id], |r| r.get(0)
        ).unwrap_or(0.0);

        let today_pnl: f64 = conn.query_row(
            "SELECT COALESCE(SUM(pnl), 0) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND date(exit_time) = ?2",
            rusqlite::params![account_id, today], |r| r.get(0)
        ).unwrap_or(0.0);

        let month_trades: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND date(exit_time) >= ?2",
            rusqlite::params![account_id, month_start], |r| r.get(0)
        ).unwrap_or(0);

        let month_pnl: f64 = conn.query_row(
            "SELECT COALESCE(SUM(pnl), 0) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND date(exit_time) >= ?2",
            rusqlite::params![account_id, month_start], |r| r.get(0)
        ).unwrap_or(0.0);

        let open_positions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_log WHERE account_id = ?1 AND status = 'open'",
            [account_id], |r| r.get(0)
        ).unwrap_or(0);

        let pending_plans: i64 = conn.query_row(
            "SELECT COUNT(*) FROM trade_plan WHERE account_id = ?1 AND status = 'planned'",
            [account_id], |r| r.get(0)
        ).unwrap_or(0);

        Ok(DashboardStats {
            balance: account.map(|a| a.0).unwrap_or(0.0),
            today_pnl, total_pnl, total_trades, win_trades, loss_trades,
            win_rate: if total_trades > 0 { win_trades as f64 / total_trades as f64 } else { 0.0 },
            month_trades, month_pnl, open_positions, pending_plans,
        })
    }

    pub fn get_pnl_trend(state: &State<'_, DbState>, account_id: &str, days: i64) -> Result<Vec<PnlTrend>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;

        let start_date = chrono::Local::now() - chrono::Duration::days(days);
        let start_str = start_date.format("%Y-%m-%d").to_string();

        let mut stmt = conn.prepare(
            "SELECT date(exit_time) as d, COALESCE(SUM(pnl), 0) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND date(exit_time) >= ?2 GROUP BY date(exit_time) ORDER BY d"
        ).map_err(|e| AppError::Database(format!("{}", e)))?;

        let rows = stmt.query_map(rusqlite::params![account_id, start_str], |row| {
            Ok(PnlTrend {
                date: row.get(0)?,
                pnl: row.get(1)?,
                cumulative_pnl: 0.0,
            })
        }).map_err(|e| AppError::Database(format!("{}", e)))?;

        let mut trends: Vec<PnlTrend> = rows.filter_map(|r| r.ok()).collect();

        let mut cumulative = 0.0;
        for t in &mut trends {
            cumulative += t.pnl;
            t.cumulative_pnl = cumulative;
        }

        Ok(trends)
    }

    pub fn get_symbol_pnl(state: &State<'_, DbState>, account_id: &str) -> Result<Vec<SymbolPnl>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;

        let mut stmt = conn.prepare(
            "SELECT symbol, COALESCE(MAX(name), symbol) as name, COUNT(*) as cnt, COALESCE(SUM(pnl), 0) as pnl, SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END) as wins FROM trade_log WHERE account_id = ?1 AND status = 'closed' GROUP BY symbol ORDER BY pnl DESC"
        ).map_err(|e| AppError::Database(format!("{}", e)))?;

        let rows = stmt.query_map([account_id], |row| {
            let count: i64 = row.get(2)?;
            let wins: i64 = row.get(4)?;
            Ok(SymbolPnl {
                symbol: row.get(0)?,
                name: row.get(1)?,
                trade_count: count,
                total_pnl: row.get(3)?,
                win_rate: if count > 0 { wins as f64 / count as f64 } else { 0.0 },
            })
        }).map_err(|e| AppError::Database(format!("{}", e)))?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
