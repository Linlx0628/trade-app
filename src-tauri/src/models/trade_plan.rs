use serde::{Deserialize, Serialize};

/// 交易计划实体 - 对应数据库 trade_plan 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePlan {
    pub id: String,
    pub account_id: String,
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default)]
    pub entry_price: f64,
    #[serde(default)]
    pub stop_loss: f64,
    #[serde(default)]
    pub take_profit: f64,
    #[serde(default)]
    pub lots: f64,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default = "default_empty_array")]
    pub tags: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_empty_array")]
    pub images: String,
    #[serde(default)]
    pub planned_at: String,
    pub created_at: String,
    pub updated_at: String,
}

fn default_direction() -> String {
    "long".to_string()
}

fn default_market_type() -> String {
    "futures".to_string()
}

fn default_status() -> String {
    "planned".to_string()
}

fn default_empty_array() -> String {
    "[]".to_string()
}

/// 创建交易计划 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradePlanDto {
    pub account_id: String,
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default)]
    pub entry_price: f64,
    #[serde(default)]
    pub stop_loss: f64,
    #[serde(default)]
    pub take_profit: f64,
    #[serde(default)]
    pub lots: f64,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub planned_at: String,
}

/// 更新交易计划 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTradePlanDto {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lots: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_at: Option<String>,
}
