use crate::db::trade_log_repo;
use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_log::{CreateTradeLogDto, TradeLog, UpdateTradeLogDto};

pub struct TradeLogService;

impl TradeLogService {
    pub fn get_by_account(state: &DbState, account_id: &str, status: Option<&str>) -> Result<Vec<TradeLog>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_log_repo::find_by_account(&conn, account_id, status)
    }

    pub fn get_by_id(state: &DbState, id: &str) -> Result<TradeLog, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_log_repo::find_by_id(&conn, id)
    }

    pub fn create(state: &DbState, dto: &CreateTradeLogDto) -> Result<TradeLog, AppError> {
        if dto.symbol.trim().is_empty() {
            return Err(AppError::Validation("交易品种不能为空".to_string()));
        }
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_log_repo::create(&conn, dto)
    }

    pub fn update(state: &DbState, dto: &UpdateTradeLogDto) -> Result<TradeLog, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_log_repo::update(&conn, dto)
    }

    pub fn delete(state: &DbState, id: &str) -> Result<(), AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_log_repo::delete(&conn, id)
    }
}
