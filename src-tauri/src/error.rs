//! 统一错误处理模块
//!
//! 定义应用级别的错误类型，支持序列化以便通过 Tauri Command 返回给前端。

use serde::Serialize;

/// 应用统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 数据库操作错误
    #[error("数据库错误: {0}")]
    Database(String),

    /// 记录未找到
    #[error("记录未找到: {0}")]
    NotFound(String),

    /// 数据验证错误
    #[error("验证错误: {0}")]
    Validation(String),

    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// 实现 Serialize 以便错误可以作为 Tauri Command 返回值
/// 将错误序列化为字符串形式
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// 定义 Tauri Command 错误返回类型别名
pub type CommandResult<T> = Result<T, AppError>;

/// 从 rusqlite 错误转换为 AppError
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("查询未返回结果".to_string())
            }
            _ => AppError::Database(err.to_string()),
        }
    }
}
