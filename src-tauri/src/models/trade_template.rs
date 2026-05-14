use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTemplate {
    pub id: String,
    pub account_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default = "default_empty_array")]
    pub tags: String,
    #[serde(default)]
    pub stop_loss_ratio: f64,
    #[serde(default)]
    pub take_profit_ratio: f64,
    #[serde(default)]
    pub default_lots: f64,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub usage_count: i64,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradeTemplateDto {
    pub account_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub symbol: String,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default = "default_market_type")]
    pub market_type: String,
    #[serde(default)]
    pub strategy: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub stop_loss_ratio: f64,
    #[serde(default)]
    pub take_profit_ratio: f64,
    #[serde(default)]
    pub default_lots: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTradeTemplateDto {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_loss_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub take_profit_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_lots: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_pinned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateFromPlanDto {
    pub plan_id: String,
    pub template_name: String,
    #[serde(default)]
    pub template_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanFromTemplateDto {
    pub template_id: String,
    pub account_id: String,
    pub entry_price: f64,
    #[serde(default)]
    pub planned_at: String,
    #[serde(default)]
    pub actual_lots: f64,
}

fn default_direction() -> String { "long".to_string() }
fn default_market_type() -> String { "futures".to_string() }
fn default_empty_array() -> String { "[]".to_string() }
