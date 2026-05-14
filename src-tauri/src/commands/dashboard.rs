use crate::db::DbState;
use crate::error::AppError;
use crate::services::dashboard_service::{DashboardStats, PnlTrend, SymbolPnl};
use crate::services::DashboardService;
use tauri::State;

#[tauri::command]
pub fn get_dashboard_stats(
    state: State<'_, DbState>,
    account_id: String,
) -> Result<DashboardStats, AppError> {
    DashboardService::get_stats(&state, &account_id)
}

#[tauri::command]
pub fn get_pnl_trend(
    state: State<'_, DbState>,
    account_id: String,
    days: Option<i64>,
) -> Result<Vec<PnlTrend>, AppError> {
    DashboardService::get_pnl_trend(&state, &account_id, days.unwrap_or(30))
}

#[tauri::command]
pub fn get_symbol_pnl(
    state: State<'_, DbState>,
    account_id: String,
) -> Result<Vec<SymbolPnl>, AppError> {
    DashboardService::get_symbol_pnl(&state, &account_id)
}
