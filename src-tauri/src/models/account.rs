//! 账户数据模型
//!
//! 定义账户实体及其创建/更新 DTO

use serde::{Deserialize, Serialize};

/// 市场类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    /// 期货
    Futures,
    /// 股票
    Stock,
}

impl Default for MarketType {
    fn default() -> Self {
        MarketType::Futures
    }
}

impl MarketType {
    /// 转换为数据库存储字符串
    pub fn as_str(&self) -> &str {
        match self {
            MarketType::Futures => "futures",
            MarketType::Stock => "stock",
        }
    }

    /// 从数据库字符串解析
    pub fn from_str(s: &str) -> Self {
        match s {
            "stock" => MarketType::Stock,
            _ => MarketType::Futures,
        }
    }
}

/// 账户实体 - 对应数据库 account 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// 唯一标识 (UUID)
    pub id: String,
    /// 账户名称
    pub name: String,
    /// 账户余额
    pub balance: f64,
    /// 风险比例（如 0.02 表示 2%）
    pub risk_ratio: f64,
    /// 每点价值（期货合约乘数）
    pub point_value: f64,
    /// 市场类型
    #[serde(default = "default_market_type")]
    pub market_type: String,
    /// 账户描述
    #[serde(default)]
    pub description: String,
    /// 是否激活
    #[serde(default = "default_true")]
    pub is_active: bool,
    /// 创建时间 (RFC3339)
    pub created_at: String,
    /// 更新时间 (RFC3339)
    pub updated_at: String,
}

fn default_market_type() -> String {
    "futures".to_string()
}

fn default_true() -> bool {
    true
}

/// 创建账户 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountDto {
    /// 账户名称（必填）
    pub name: String,
    /// 账户余额（默认 0.0）
    #[serde(default)]
    pub balance: f64,
    /// 风险比例（默认 0.02）
    #[serde(default = "default_risk_ratio")]
    pub risk_ratio: f64,
    /// 每点价值（默认 1.0）
    #[serde(default = "default_point_value")]
    pub point_value: f64,
    /// 市场类型（默认 futures）
    #[serde(default = "default_market_type")]
    pub market_type: String,
    /// 账户描述
    #[serde(default)]
    pub description: String,
}

fn default_risk_ratio() -> f64 {
    0.02
}

fn default_point_value() -> f64 {
    1.0
}

/// 更新账户 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountDto {
    /// 账户 ID（必填）
    pub id: String,
    /// 账户名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 账户余额
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<f64>,
    /// 风险比例
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_ratio: Option<f64>,
    /// 每点价值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point_value: Option<f64>,
    /// 市场类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type: Option<String>,
    /// 账户描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 是否激活
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// 账户统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStats {
    /// 账户 ID
    pub account_id: String,
    /// 账户名称
    pub account_name: String,
    /// 当前余额
    pub balance: f64,
    /// 净值（余额 + 未平仓盈亏）
    pub net_value: f64,
    /// 总盈亏
    pub total_pnl: f64,
    /// 总手续费
    pub total_commission: f64,
    /// 净盈亏
    pub net_pnl: f64,
    /// 总交易笔数
    pub total_trades: i64,
    /// 盈利笔数
    pub win_trades: i64,
    /// 亏损笔数
    pub loss_trades: i64,
    /// 胜率
    pub win_rate: f64,
    /// 平均盈利
    pub avg_profit: f64,
    /// 平均亏损
    pub avg_loss: f64,
    /// 盈亏比
    pub profit_factor: f64,
    /// 最大单笔盈利
    pub max_profit: f64,
    /// 最大单笔亏损
    pub max_loss: f64,
}
