use crate::db::DbState;
use crate::error::AppError;
use crate::models::trade_plan::{CreateTradePlanDto, TradePlan};
use crate::models::trade_template::{
    CreatePlanFromTemplateDto, CreateTemplateFromPlanDto, CreateTradeTemplateDto,
    TradeTemplate, UpdateTradeTemplateDto,
};
use crate::services::TradePlanService;
use tauri::State;

pub struct TradeTemplateService;

impl TradeTemplateService {
    pub fn get_by_account(state: &State<'_, DbState>, account_id: &str) -> Result<Vec<TradeTemplate>, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        crate::db::trade_template_repo::find_by_account(&conn, account_id)
    }

    pub fn get_by_id(state: &State<'_, DbState>, id: &str) -> Result<TradeTemplate, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        crate::db::trade_template_repo::find_by_id(&conn, id)
    }

    pub fn create(state: &State<'_, DbState>, dto: &CreateTradeTemplateDto) -> Result<TradeTemplate, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        crate::db::trade_template_repo::insert(&conn, dto)
    }

    pub fn update(state: &State<'_, DbState>, dto: &UpdateTradeTemplateDto) -> Result<TradeTemplate, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        crate::db::trade_template_repo::update(&conn, dto)
    }

    pub fn delete(state: &State<'_, DbState>, id: &str) -> Result<(), AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
        crate::db::trade_template_repo::delete(&conn, id)
    }

    /// 从已有交易计划创建模板
    pub fn create_from_plan(
        state: &State<'_, DbState>,
        dto: &CreateTemplateFromPlanDto,
    ) -> Result<TradeTemplate, AppError> {
        let plan = TradePlanService::get_by_id(state, &dto.plan_id)?;

        // 计算止损止盈比例（相对于入场价）
        let stop_loss_ratio = if plan.entry_price > 0.0 {
            ((plan.entry_price - plan.stop_loss) / plan.entry_price).abs()
        } else {
            0.0
        };
        let take_profit_ratio = if plan.entry_price > 0.0 {
            ((plan.take_profit - plan.entry_price) / plan.entry_price).abs()
        } else {
            0.0
        };

        let template_dto = CreateTradeTemplateDto {
            account_id: plan.account_id.clone(),
            name: dto.template_name.clone(),
            description: dto.template_description.clone(),
            symbol: plan.symbol.clone(),
            direction: plan.direction.clone(),
            market_type: plan.market_type.clone(),
            strategy: plan.strategy.clone(),
            tags: serde_json::from_str(&plan.tags).unwrap_or_default(),
            stop_loss_ratio,
            take_profit_ratio,
            default_lots: plan.lots,
            notes: plan.notes.clone(),
        };

        Self::create(state, &template_dto)
    }

    /// 从模板创建交易计划
    pub fn create_plan_from_template(
        state: &State<'_, DbState>,
        dto: &CreatePlanFromTemplateDto,
    ) -> Result<TradePlan, AppError> {
        let template = Self::get_by_id(state, &dto.template_id)?;

        // 根据方向和比例计算止损止盈价格
        let dir_mult = if template.direction == "long" { 1.0 } else { -1.0 };
        let stop_loss = dto.entry_price - dto.entry_price * template.stop_loss_ratio * dir_mult;
        let take_profit = dto.entry_price + dto.entry_price * template.take_profit_ratio * dir_mult;
        let lots = if dto.actual_lots > 0.0 { dto.actual_lots } else { template.default_lots };

        let plan_dto = CreateTradePlanDto {
            account_id: dto.account_id.clone(),
            symbol: template.symbol.clone(),
            name: String::new(),
            direction: template.direction.clone(),
            market_type: template.market_type.clone(),
            entry_price: dto.entry_price,
            stop_loss,
            take_profit,
            lots,
            strategy: template.strategy.clone(),
            notes: template.notes.clone(),
            tags: serde_json::from_str(&template.tags).unwrap_or_default(),
            planned_at: if dto.planned_at.is_empty() { chrono::Utc::now().to_rfc3339() } else { dto.planned_at.clone() },
        };

        // 增加模板使用次数
        {
            let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
            crate::db::trade_template_repo::increment_usage(&conn, &dto.template_id)?;
        }

        TradePlanService::create(state, &plan_dto)
    }
}
