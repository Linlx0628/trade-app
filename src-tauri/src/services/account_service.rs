//! 账户服务层
//!
//! 封装账户相关的业务逻辑，调用 Repository 层进行数据操作

use crate::db::account_repo;
use crate::db::DbState;
use crate::error::AppError;
use crate::models::{Account, AccountStats, CreateAccountDto, UpdateAccountDto};

/// 账户服务
///
/// 提供账户 CRUD 和统计查询的业务逻辑
pub struct AccountService;

impl AccountService {
    /// 获取所有账户列表
    pub fn get_all(state: &DbState) -> Result<Vec<Account>, AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::find_all(&conn)
    }

    /// 根据 ID 获取单个账户
    pub fn get_by_id(state: &DbState, id: &str) -> Result<Account, AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::find_by_id(&conn, id)
    }

    /// 创建新账户
    pub fn create(state: &DbState, dto: &CreateAccountDto) -> Result<Account, AppError> {
        // 业务验证
        if dto.risk_ratio <= 0.0 || dto.risk_ratio > 1.0 {
            return Err(AppError::Validation(
                "风险比例必须在 0 到 1 之间".to_string(),
            ));
        }
        if dto.point_value <= 0.0 {
            return Err(AppError::Validation("每点价值必须大于 0".to_string()));
        }
        if dto.balance < 0.0 {
            return Err(AppError::Validation("账户余额不能为负数".to_string()));
        }

        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::create(&conn, dto)
    }

    /// 更新账户信息
    pub fn update(state: &DbState, dto: &UpdateAccountDto) -> Result<Account, AppError> {
        // 业务验证
        if let Some(risk_ratio) = dto.risk_ratio {
            if risk_ratio <= 0.0 || risk_ratio > 1.0 {
                return Err(AppError::Validation(
                    "风险比例必须在 0 到 1 之间".to_string(),
                ));
            }
        }
        if let Some(point_value) = dto.point_value {
            if point_value <= 0.0 {
                return Err(AppError::Validation("每点价值必须大于 0".to_string()));
            }
        }
        if let Some(balance) = dto.balance {
            if balance < 0.0 {
                return Err(AppError::Validation("账户余额不能为负数".to_string()));
            }
        }

        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::update(&conn, dto)
    }

    /// 删除账户
    ///
    /// 关联的交易计划和日志会通过数据库外键级联删除
    pub fn delete(state: &DbState, id: &str) -> Result<(), AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::delete(&conn, id)
    }

    /// 获取账户统计数据
    pub fn get_stats(state: &DbState, account_id: &str) -> Result<AccountStats, AppError> {
        let conn = state
            .conn
            .lock()
            .map_err(|e| AppError::Database(format!("获取数据库连接失败: {}", e)))?;
        account_repo::find_stats(&conn, account_id)
    }
}
