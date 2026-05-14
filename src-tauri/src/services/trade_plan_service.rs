use crate::db::trade_plan_repo;
use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_plan::{CreateTradePlanDto, TradePlan, UpdateTradePlanDto};

pub struct TradePlanService;

impl TradePlanService {
    pub fn get_by_account(
        state: &DbState,
        account_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<TradePlan>, AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_plan_repo::find_by_account(&conn, account_id, status)
    }

    pub fn get_by_id(state: &DbState, id: &str) -> Result<TradePlan, AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_plan_repo::find_by_id(&conn, id)
    }

    pub fn create(state: &DbState, dto: &CreateTradePlanDto) -> Result<TradePlan, AppError> {
        if dto.symbol.trim().is_empty() {
            return Err(AppError::Validation("交易品种不能为空".to_string()));
        }
        if !["long", "short"].contains(&dto.direction.as_str()) {
            return Err(AppError::Validation("交易方向必须为 long 或 short".to_string()));
        }
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_plan_repo::create(&conn, dto)
    }

    pub fn update(state: &DbState, dto: &UpdateTradePlanDto) -> Result<TradePlan, AppError> {
        if let Some(ref direction) = dto.direction {
            if !["long", "short"].contains(&direction.as_str()) {
                return Err(AppError::Validation("交易方向必须为 long 或 short".to_string()));
            }
        }
        if let Some(ref status) = dto.status {
            if !["planned", "executing", "completed", "cancelled"].contains(&status.as_str()) {
                return Err(AppError::Validation(
                    "状态必须为 planned, executing, completed 或 cancelled".to_string(),
                ));
            }
        }
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_plan_repo::update(&conn, dto)
    }

    pub fn delete(state: &DbState, id: &str) -> Result<(), AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_plan_repo::delete(&conn, id)
    }
}
