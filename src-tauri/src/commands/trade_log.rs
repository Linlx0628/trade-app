use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_log::{CreateTradeLogDto, TradeLog, UpdateTradeLogDto};
use crate::services::TradeLogService;
use tauri::State;

#[tauri::command]
pub fn get_trade_logs(state: State<'_, DbState>, account_id: String, status: Option<String>) -> Result<Vec<TradeLog>, AppError> {
    TradeLogService::get_by_account(&state, &account_id, status.as_deref())
}

#[tauri::command]
pub fn get_trade_log(state: State<'_, DbState>, id: String) -> Result<TradeLog, AppError> {
    TradeLogService::get_by_id(&state, &id)
}

#[tauri::command]
pub fn create_trade_log(state: State<'_, DbState>, dto: CreateTradeLogDto) -> Result<TradeLog, AppError> {
    TradeLogService::create(&state, &dto)
}

#[tauri::command]
pub fn update_trade_log(state: State<'_, DbState>, dto: UpdateTradeLogDto) -> Result<TradeLog, AppError> {
    TradeLogService::update(&state, &dto)
}

#[tauri::command]
pub fn delete_trade_log(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    TradeLogService::delete(&state, &id)
}
