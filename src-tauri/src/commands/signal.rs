use crate::db::DbState;
use crate::error::AppError;
use crate::models::signal_alert::{ChanlunAnalysis, CreateSignalAlertDto, SignalAlert};
use crate::services::chanlun_service::ChanlunService;
use crate::services::market_data_service::MarketDataService;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn analyze_chanlun(symbol: String, period: Option<String>) -> Result<ChanlunAnalysis, AppError> {
    let p = period.unwrap_or_else(|| "day".to_string());
    let klines = MarketDataService::get_kline(&symbol, &p, 200).await?;
    Ok(ChanlunService::analyze(&klines, &symbol))
}

#[tauri::command]
pub fn get_signal_alerts(state: State<'_, DbState>, account_id: String) -> Result<Vec<SignalAlert>, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, symbol, alert_type, condition_value, description, is_active, is_triggered, triggered_at, created_at, updated_at FROM signal_alert WHERE account_id = ?1 ORDER BY created_at DESC"
    )?;

    let alerts = stmt.query_map(params![account_id], |row| {
        Ok(SignalAlert {
            id: row.get(0)?,
            account_id: row.get(1)?,
            symbol: row.get(2)?,
            alert_type: row.get(3)?,
            condition_value: row.get(4)?,
            description: row.get(5)?,
            is_active: row.get::<_, i32>(6)? != 0,
            is_triggered: row.get::<_, i32>(7)? != 0,
            triggered_at: row.get(8)?,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(alerts)
}

#[tauri::command]
pub fn create_signal_alert(state: State<'_, DbState>, dto: CreateSignalAlertDto) -> Result<SignalAlert, AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO signal_alert (id, account_id, symbol, alert_type, condition_value, description, is_active, is_triggered, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 0, ?7, ?8)",
        params![id, dto.account_id, dto.symbol, dto.alert_type, dto.condition_value, dto.description, now, now],
    )?;

    Ok(SignalAlert {
        id,
        account_id: dto.account_id,
        symbol: dto.symbol,
        alert_type: dto.alert_type,
        condition_value: dto.condition_value,
        description: dto.description,
        is_active: true,
        is_triggered: false,
        triggered_at: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_signal_alert(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
    conn.execute("DELETE FROM signal_alert WHERE id = ?1", params![id])?;
    Ok(())
}
