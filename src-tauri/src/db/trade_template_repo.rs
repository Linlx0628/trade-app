use crate::error::AppError;
use crate::models::trade_template::{CreateTradeTemplateDto, TradeTemplate, UpdateTradeTemplateDto};
use rusqlite::Connection;

fn map_template(row: &rusqlite::Row<'_>) -> Result<TradeTemplate, rusqlite::Error> {
    Ok(TradeTemplate {
        id: row.get(0)?,
        account_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        symbol: row.get(4)?,
        direction: row.get(5)?,
        market_type: row.get(6)?,
        strategy: row.get(7)?,
        tags: row.get(8)?,
        stop_loss_ratio: row.get(9)?,
        take_profit_ratio: row.get(10)?,
        default_lots: row.get(11)?,
        notes: row.get(12)?,
        usage_count: row.get(13)?,
        sort_order: row.get(14)?,
        is_pinned: row.get::<_, i64>(15)? != 0,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

const SELECT_COLUMNS: &str = "id, account_id, name, description, symbol, direction, market_type, strategy, tags, stop_loss_ratio, take_profit_ratio, default_lots, notes, usage_count, sort_order, is_pinned, created_at, updated_at";

pub fn find_by_account(
    conn: &Connection,
    account_id: &str,
) -> Result<Vec<TradeTemplate>, AppError> {
    let sql = format!(
        "SELECT {} FROM trade_template WHERE account_id = ?1 OR account_id = '' ORDER BY is_pinned DESC, usage_count DESC, sort_order ASC, created_at DESC",
        SELECT_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;
    let templates = stmt
        .query_map([account_id], |row| map_template(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(templates)
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<TradeTemplate, AppError> {
    let sql = format!("SELECT {} FROM trade_template WHERE id = ?1", SELECT_COLUMNS);
    conn.query_row(&sql, [id], |row| map_template(row))
        .map_err(|e| AppError::Database(format!("模板不存在: {}", e)))
}

pub fn insert(conn: &Connection, dto: &CreateTradeTemplateDto) -> Result<TradeTemplate, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&dto.tags).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO trade_template (id, account_id, name, description, symbol, direction, market_type, strategy, tags, stop_loss_ratio, take_profit_ratio, default_lots, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        rusqlite::params![
            id, dto.account_id, dto.name, dto.description,
            dto.symbol, dto.direction, dto.market_type, dto.strategy, tags_json,
            dto.stop_loss_ratio, dto.take_profit_ratio, dto.default_lots, dto.notes,
            now, now
        ],
    )?;

    find_by_id(conn, &id)
}

pub fn update(conn: &Connection, dto: &UpdateTradeTemplateDto) -> Result<TradeTemplate, AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let mut sets = vec!["updated_at = ?1".to_string()];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    let mut idx = 2u32;
    if let Some(ref v) = dto.name { sets.push(format!("name = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.description { sets.push(format!("description = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.symbol { sets.push(format!("symbol = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.direction { sets.push(format!("direction = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.market_type { sets.push(format!("market_type = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.strategy { sets.push(format!("strategy = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(ref v) = dto.tags { sets.push(format!("tags = ?{}", idx)); params.push(Box::new(serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()))); idx += 1; }
    if let Some(v) = dto.stop_loss_ratio { sets.push(format!("stop_loss_ratio = ?{}", idx)); params.push(Box::new(v)); idx += 1; }
    if let Some(v) = dto.take_profit_ratio { sets.push(format!("take_profit_ratio = ?{}", idx)); params.push(Box::new(v)); idx += 1; }
    if let Some(v) = dto.default_lots { sets.push(format!("default_lots = ?{}", idx)); params.push(Box::new(v)); idx += 1; }
    if let Some(ref v) = dto.notes { sets.push(format!("notes = ?{}", idx)); params.push(Box::new(v.clone())); idx += 1; }
    if let Some(v) = dto.sort_order { sets.push(format!("sort_order = ?{}", idx)); params.push(Box::new(v)); idx += 1; }
    if let Some(v) = dto.is_pinned { sets.push(format!("is_pinned = ?{}", idx)); params.push(Box::new(v as i64)); idx += 1; }

    params.push(Box::new(dto.id.clone()));
    let sql = format!("UPDATE trade_template SET {} WHERE id = ?{}", sets.join(", "), idx);
    conn.execute(&sql, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())))?;

    find_by_id(conn, &dto.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute("DELETE FROM trade_template WHERE id = ?1", [id])?;
    Ok(())
}

pub fn increment_usage(conn: &Connection, id: &str) -> Result<(), AppError> {
    conn.execute(
        "UPDATE trade_template SET usage_count = usage_count + 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().to_rfc3339(), id],
    )?;
    Ok(())
}
