use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_plan::TradePlan;
use crate::models::trade_template::{
    CreatePlanFromTemplateDto, CreateTemplateFromPlanDto, CreateTradeTemplateDto,
    TradeTemplate, UpdateTradeTemplateDto,
};
use crate::services::TradeTemplateService;
use tauri::State;

#[tauri::command]
pub fn get_trade_templates(
    state: State<'_, DbState>,
    account_id: String,
) -> Result<Vec<TradeTemplate>, AppError> {
    TradeTemplateService::get_by_account(&state, &account_id)
}

#[tauri::command]
pub fn get_trade_template(
    state: State<'_, DbState>,
    id: String,
) -> Result<TradeTemplate, AppError> {
    TradeTemplateService::get_by_id(&state, &id)
}

#[tauri::command]
pub fn create_trade_template(
    state: State<'_, DbState>,
    dto: CreateTradeTemplateDto,
) -> Result<TradeTemplate, AppError> {
    TradeTemplateService::create(&state, &dto)
}

#[tauri::command]
pub fn update_trade_template(
    state: State<'_, DbState>,
    dto: UpdateTradeTemplateDto,
) -> Result<TradeTemplate, AppError> {
    TradeTemplateService::update(&state, &dto)
}

#[tauri::command]
pub fn delete_trade_template(
    state: State<'_, DbState>,
    id: String,
) -> Result<(), AppError> {
    TradeTemplateService::delete(&state, &id)
}

#[tauri::command]
pub fn create_template_from_plan(
    state: State<'_, DbState>,
    dto: CreateTemplateFromPlanDto,
) -> Result<TradeTemplate, AppError> {
    TradeTemplateService::create_from_plan(&state, &dto)
}

#[tauri::command]
pub fn create_plan_from_template(
    state: State<'_, DbState>,
    dto: CreatePlanFromTemplateDto,
) -> Result<TradePlan, AppError> {
    TradeTemplateService::create_plan_from_template(&state, &dto)
}
