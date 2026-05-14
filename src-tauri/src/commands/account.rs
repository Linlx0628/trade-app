//! 账户相关 Tauri Commands
//!
//! 提供账户管理和交易计算的前端调用接口

use crate::db::DbState;
use crate::error::AppError;
use crate::models::{Account, AccountStats, CreateAccountDto, UpdateAccountDto};
use crate::services::{AccountService, CalculationService};
use crate::services::calculation_service::CalculationResult;
use tauri::State;

/// 获取所有账户列表
#[tauri::command]
pub fn get_accounts(state: State<'_, DbState>) -> Result<Vec<Account>, AppError> {
    AccountService::get_all(&state)
}

/// 根据 ID 获取单个账户
#[tauri::command]
pub fn get_account(state: State<'_, DbState>, id: String) -> Result<Account, AppError> {
    AccountService::get_by_id(&state, &id)
}

/// 创建新账户
#[tauri::command]
pub fn create_account(
    state: State<'_, DbState>,
    dto: CreateAccountDto,
) -> Result<Account, AppError> {
    AccountService::create(&state, &dto)
}

/// 更新账户信息
#[tauri::command]
pub fn update_account(
    state: State<'_, DbState>,
    dto: UpdateAccountDto,
) -> Result<Account, AppError> {
    AccountService::update(&state, &dto)
}

/// 删除账户
#[tauri::command]
pub fn delete_account(state: State<'_, DbState>, id: String) -> Result<(), AppError> {
    AccountService::delete(&state, &id)
}

/// 获取账户统计数据
///
/// 返回指定账户的交易统计信息，包括胜率、盈亏比等
#[tauri::command]
pub fn get_account_stats(
    state: State<'_, DbState>,
    account_id: String,
) -> Result<AccountStats, AppError> {
    AccountService::get_stats(&state, &account_id)
}

/// 执行交易计算
///
/// 根据账户参数和交易价格计算风险指标
#[tauri::command]
pub fn calculate_trade(
    state: State<'_, DbState>,
    account_id: String,
    entry_price: f64,
    stop_loss: f64,
    take_profit: f64,
) -> Result<CalculationResult, AppError> {
    // 获取账户信息
    let account = AccountService::get_by_id(&state, &account_id)?;

    // 执行计算
    let result = CalculationService::calculate(
        account.balance,
        account.risk_ratio,
        account.point_value,
        entry_price,
        stop_loss,
        take_profit,
    );

    Ok(result)
}
