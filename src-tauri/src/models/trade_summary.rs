use serde::{Deserialize, Serialize};

/// 交易总结实体 - 对应数据库 trade_summary 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSummary {
    pub id: String,
    pub account_id: String,
    #[serde(default = "default_daily")]
    pub summary_type: String,
    pub summary_date: String,
    #[serde(default)]
    pub total_trades: i32,
    #[serde(default)]
    pub win_trades: i32,
    #[serde(default)]
    pub loss_trades: i32,
    #[serde(default)]
    pub total_pnl: f64,
    #[serde(default)]
    pub total_commission: f64,
    #[serde(default)]
    pub net_pnl: f64,
    #[serde(default)]
    pub win_rate: f64,
    #[serde(default)]
    pub avg_profit: f64,
    #[serde(default)]
    pub avg_loss: f64,
    #[serde(default)]
    pub profit_factor: f64,
    #[serde(default)]
    pub max_profit: f64,
    #[serde(default)]
    pub max_loss: f64,
    #[serde(default = "default_emotion")]
    pub emotion_score: i32,
    #[serde(default)]
    pub market_view: String,
    #[serde(default)]
    pub lessons: String,
    #[serde(default)]
    pub improvement: String,
    #[serde(default = "default_empty_array")]
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

fn default_daily() -> String { "daily".to_string() }
fn default_emotion() -> i32 { 5 }
fn default_empty_array() -> String { "[]".to_string() }

/// 创建交易总结 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradeSummaryDto {
    pub account_id: String,
    #[serde(default = "default_daily")]
    pub summary_type: String,
    pub summary_date: String,
    #[serde(default)]
    pub emotion_score: i32,
    #[serde(default)]
    pub market_view: String,
    #[serde(default)]
    pub lessons: String,
    #[serde(default)]
    pub improvement: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 更新交易总结 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTradeSummaryDto {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion_score: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lessons: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub improvement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}
