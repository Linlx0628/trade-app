use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_plan::{CreateTradePlanDto, TradePlan, UpdateTradePlanDto};
use crate::services::TradePlanService;
use tauri::State;

#[tauri::command]
pub fn get_trade_plans(
    state: State<'_, DbState>,
    account_id: String,
    status: Option<String>,
) -> Result<Vec<TradePlan>, AppError> {
    TradePlanService::get_by_account(&state, &account_id, status.as_deref())
}

#[tauri::command]
pub fn get_trade_plan(state: State<'_, DbState>, id: String) -> Result<TradePlan, AppError> {
    TradePlanService::get_by_id(&state, &id)
}

#[tauri::command]
pub fn create_trade_plan(
    state: State<'_, DbState>,
    dto: CreateTradePlanDto,
) -> Result<TradePlan, AppError> {
    TradePlanService::create(&state, &dto)
}

#[tauri::command]
pub fn update_trade_plan(
    state: State<'_, DbState>,
    dto: UpdateTradePlanDto,
) -> Result<TradePlan, AppError> {
    TradePlanService::update(&state, &dto)
}

#[tauri::command]
pub fn delete_trade_plan(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    TradePlanService::delete(&state, &id)
}
