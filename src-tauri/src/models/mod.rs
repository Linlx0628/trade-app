//! 数据模型模块

pub mod account;
pub mod trade_log;
pub mod trade_plan;
pub mod trade_summary;
pub mod trade_template;

pub use account::{Account, AccountStats, CreateAccountDto, UpdateAccountDto};
pub use trade_log::{CreateTradeLogDto, TradeLog, UpdateTradeLogDto};
pub use trade_plan::{CreateTradePlanDto, TradePlan, UpdateTradePlanDto};
pub use trade_summary::{CreateTradeSummaryDto, TradeSummary, UpdateTradeSummaryDto};
pub use trade_template::{
    CreatePlanFromTemplateDto, CreateTemplateFromPlanDto, CreateTradeTemplateDto,
    TradeTemplate, UpdateTradeTemplateDto,
};
