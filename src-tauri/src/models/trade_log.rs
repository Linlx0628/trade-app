use serde::{Deserialize, Serialize};

/// 交易日志实体 - 对应数据库 trade_log 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLog {
    pub id: String,
    pub account_id: String,
    pub plan_id: Option<String>,
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default)]
    pub entry_price: f64,
    #[serde(default)]
    pub exit_price: f64,
    #[serde(default)]
    pub stop_loss: f64,
    #[serde(default)]
    pub lots: f64,
    #[serde(default)]
    pub pnl: f64,
    #[serde(default)]
    pub pnl_points: f64,
    #[serde(default)]
    pub commission: f64,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default = "default_open_status")]
    pub status: String,
    #[serde(default)]
    pub entry_time: String,
    #[serde(default)]
    pub exit_time: String,
    #[serde(default = "default_empty_array")]
    pub tags: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_empty_array")]
    pub images: String,
    #[serde(default)]
    pub ai_feedback: String,
    #[serde(default)]
    pub emotion_before: String,
    #[serde(default)]
    pub emotion_after: String,
    #[serde(default = "default_confidence")]
    pub confidence: i32,
    pub created_at: String,
    pub updated_at: String,
}

fn default_direction() -> String { "long".to_string() }
fn default_market_type() -> String { "futures".to_string() }
fn default_open_status() -> String { "open".to_string() }
fn default_empty_array() -> String { "[]".to_string() }
fn default_confidence() -> i32 { 5 }

/// 创建交易日志 DTO
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateTradeLogDto {
    pub account_id: String,
    pub plan_id: Option<String>,
    pub symbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    pub entry_price: f64,
    #[serde(default)]
    pub exit_price: f64,
    #[serde(default)]
    pub stop_loss: f64,
    pub lots: f64,
    #[serde(default)]
    pub pnl: f64,
    #[serde(default)]
    pub pnl_points: f64,
    #[serde(default)]
    pub commission: f64,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default = "default_open_status")]
    pub status: String,
    #[serde(default)]
    pub entry_time: String,
    #[serde(default)]
    pub exit_time: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub emotion_before: String,
    #[serde(default = "default_confidence")]
    pub confidence: i32,
}

/// 更新交易日志 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTradeLogDto {
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
    pub exit_price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lots: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnl: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pnl_points: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commission: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emotion_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<i32>,
}
