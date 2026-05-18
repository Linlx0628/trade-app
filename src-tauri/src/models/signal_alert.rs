use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAlert {
    pub id: String,
    pub account_id: String,
    pub symbol: String,
    pub alert_type: String,
    pub condition_value: Option<f64>,
    pub description: Option<String>,
    pub is_active: bool,
    pub is_triggered: bool,
    pub triggered_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSignalAlertDto {
    pub account_id: String,
    pub symbol: String,
    pub alert_type: String,
    pub condition_value: Option<f64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fractal {
    pub index: u32,
    pub fractal_type: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bi {
    pub start_index: u32,
    pub end_index: u32,
    pub direction: String,
    pub start_value: f64,
    pub end_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pivot {
    pub start_index: u32,
    pub end_index: u32,
    pub zg: f64,
    pub zd: f64,
    pub zz: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanlunSignal {
    pub signal_type: String,
    pub index: u32,
    pub price: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanlunAnalysis {
    pub symbol: String,
    pub fractals: Vec<Fractal>,
    pub bis: Vec<Bi>,
    pub pivots: Vec<Pivot>,
    pub signals: Vec<ChanlunSignal>,
    pub current_trend: String,
}
