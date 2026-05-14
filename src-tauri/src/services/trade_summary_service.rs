use crate::db::trade_summary_repo;
use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_summary::{CreateTradeSummaryDto, TradeSummary, UpdateTradeSummaryDto};

pub struct TradeSummaryService;

impl TradeSummaryService {
    pub fn get_by_account(state: &DbState, account_id: &str, summary_type: Option<&str>) -> Result<Vec<TradeSummary>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_summary_repo::find_by_account(&conn, account_id, summary_type)
    }

    pub fn get_by_id(state: &DbState, id: &str) -> Result<TradeSummary, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_summary_repo::find_by_id(&conn, id)
    }

    pub fn create(state: &DbState, dto: &CreateTradeSummaryDto) -> Result<TradeSummary, AppError> {
        if dto.summary_date.trim().is_empty() {
            return Err(AppError::Validation("总结日期不能为空".to_string()));
        }
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_summary_repo::create_with_aggregation(&conn, dto)
    }

    pub fn update(state: &DbState, dto: &UpdateTradeSummaryDto) -> Result<TradeSummary, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_summary_repo::update(&conn, dto)
    }

    pub fn delete(state: &DbState, id: &str) -> Result<(), AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        trade_summary_repo::delete(&conn, id)
    }
}
