use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_log::CreateTradeLogDto;
use crate::models::trade_plan::{CreateTradePlanDto, TradePlan, UpdateTradePlanDto};
use crate::services::{TradeLogService, TradePlanService};
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

/// 执行交易计划：将状态改为 executing 并自动创建关联交易日志
#[tauri::command]
pub fn execute_trade_plan(
    state: State<'_, DbState>,
    plan_id: String,
    entry_time: Option<String>,
) -> Result<crate::models::trade_log::TradeLog, AppError> {
    // 获取计划
    let plan = TradePlanService::get_by_id(&state, &plan_id)?;

    if plan.status != "planned" {
        return Err(AppError::Validation("只有计划中的交易才能执行".to_string()));
    }

    // 更新计划状态为 executing
    TradePlanService::update(&state, &UpdateTradePlanDto {
        id: plan_id.clone(),
        status: Some("executing".to_string()),
        ..Default::default()
    })?;

    // 自动创建交易日志
    let now = entry_time.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let log_dto = CreateTradeLogDto {
        account_id: plan.account_id,
        plan_id: Some(plan_id),
        symbol: plan.symbol,
        name: plan.name,
        direction: plan.direction,
        entry_price: plan.entry_price,
        stop_loss: plan.stop_loss,
        lots: plan.lots,
        market_type: plan.market_type,
        status: "open".to_string(),
        entry_time: now,
        ..Default::default()
    };

    TradeLogService::create(&state, &log_dto)
}
