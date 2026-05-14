use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_summary::{CreateTradeSummaryDto, TradeSummary, UpdateTradeSummaryDto};
use crate::services::TradeSummaryService;
use tauri::State;

#[tauri::command]
pub fn get_trade_summaries(state: State<'_, DbState>, account_id: String, summary_type: Option<String>) -> Result<Vec<TradeSummary>, AppError> {
    TradeSummaryService::get_by_account(&state, &account_id, summary_type.as_deref())
}

#[tauri::command]
pub fn get_trade_summary(state: State<'_, DbState>, id: String) -> Result<TradeSummary, AppError> {
    TradeSummaryService::get_by_id(&state, &id)
}

#[tauri::command]
pub fn create_trade_summary(state: State<'_, DbState>, dto: CreateTradeSummaryDto) -> Result<TradeSummary, AppError> {
    TradeSummaryService::create(&state, &dto)
}

#[tauri::command]
pub fn update_trade_summary(state: State<'_, DbState>, dto: UpdateTradeSummaryDto) -> Result<TradeSummary, AppError> {
    TradeSummaryService::update(&state, &dto)
}

#[tauri::command]
pub fn delete_trade_summary(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    TradeSummaryService::delete(&state, &id)
}
