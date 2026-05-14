use crate::error::AppError;
use crate::models::trade_plan::{CreateTradePlanDto, TradePlan, UpdateTradePlanDto};
use rusqlite::{params, Connection};

fn map_trade_plan(row: &rusqlite::Row<'_>) -> Result<TradePlan, rusqlite::Error> {
    Ok(TradePlan {
        id: row.get(0)?,
        account_id: row.get(1)?,
        symbol: row.get(2)?,
        name: row.get(3)?,
        direction: row.get(4)?,
        entry_price: row.get(5)?,
        stop_loss: row.get(6)?,
        take_profit: row.get(7)?,
        lots: row.get(8)?,
        market_type: row.get(9)?,
        status: row.get(10)?,
        strategy: row.get(11)?,
        tags: row.get(12)?,
        notes: row.get(13)?,
        images: row.get(14)?,
        planned_at: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

const SELECT_COLUMNS: &str = "id, account_id, symbol, name, direction, entry_price, stop_loss, take_profit, lots, market_type, status, strategy, tags, notes, images, planned_at, created_at, updated_at";

pub fn find_by_account(
    conn: &Connection,
    account_id: &str,
    status: Option<&str>,
) -> Result<Vec<TradePlan>, AppError> {
    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status {
        Some(s) => (
            format!("SELECT {} FROM trade_plan WHERE account_id = ?1 AND status = ?2 ORDER BY created_at DESC", SELECT_COLUMNS),
            vec![Box::new(account_id.to_string()), Box::new(s.to_string())],
        ),
        None => (
            format!("SELECT {} FROM trade_plan WHERE account_id = ?1 ORDER BY created_at DESC", SELECT_COLUMNS),
            vec![Box::new(account_id.to_string())],
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let plans = stmt
        .query_map(param_refs.as_slice(), |row| map_trade_plan(row))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(format!("查询交易计划失败: {}", e)))?;

    Ok(plans)
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<TradePlan, AppError> {
    let sql = format!(
        "SELECT {} FROM trade_plan WHERE id = ?1",
        SELECT_COLUMNS
    );
    conn.query_row(&sql, params![id], |row| map_trade_plan(row))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("交易计划不存在: {}", id))
            }
            _ => AppError::Database(format!("查询交易计划失败: {}", e)),
        })
}

pub fn create(conn: &Connection, dto: &CreateTradePlanDto) -> Result<TradePlan, AppError> {
    if dto.symbol.trim().is_empty() {
        return Err(AppError::Validation("交易品种不能为空".to_string()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&dto.tags)
        .map_err(|e| AppError::Serialization(e))?;

    conn.execute(
        "INSERT INTO trade_plan (id, account_id, symbol, name, direction, entry_price, stop_loss, take_profit, lots, market_type, status, strategy, tags, notes, images, planned_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, '[]', ?15, ?16, ?17)",
        params![
            id,
            dto.account_id,
            dto.symbol,
            dto.name,
            dto.direction,
            dto.entry_price,
            dto.stop_loss,
            dto.take_profit,
            dto.lots,
            dto.market_type,
            "planned",
            dto.strategy,
            tags_json,
            dto.notes,
            dto.planned_at,
            now,
            now,
        ],
    )
    .map_err(|e| AppError::Database(format!("创建交易计划失败: {}", e)))?;

    find_by_id(conn, &id)
}

pub fn update(conn: &Connection, dto: &UpdateTradePlanDto) -> Result<TradePlan, AppError> {
    let _existing = find_by_id(conn, &dto.id)?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut updates: Vec<String> = vec!["updated_at = ?".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    if let Some(ref v) = dto.symbol {
        updates.push("symbol = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.name {
        updates.push("name = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.direction {
        updates.push("direction = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(v) = dto.entry_price {
        updates.push("entry_price = ?".into());
        param_values.push(Box::new(v));
    }
    if let Some(v) = dto.stop_loss {
        updates.push("stop_loss = ?".into());
        param_values.push(Box::new(v));
    }
    if let Some(v) = dto.take_profit {
        updates.push("take_profit = ?".into());
        param_values.push(Box::new(v));
    }
    if let Some(v) = dto.lots {
        updates.push("lots = ?".into());
        param_values.push(Box::new(v));
    }
    if let Some(ref v) = dto.market_type {
        updates.push("market_type = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.status {
        updates.push("status = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.strategy {
        updates.push("strategy = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.tags {
        let json = serde_json::to_string(v).map_err(|e| AppError::Serialization(e))?;
        updates.push("tags = ?".into());
        param_values.push(Box::new(json));
    }
    if let Some(ref v) = dto.notes {
        updates.push("notes = ?".into());
        param_values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = dto.images {
        let json = serde_json::to_string(v).map_err(|e| AppError::Serialization(e))?;
        updates.push("images = ?".into());
        param_values.push(Box::new(json));
    }
    if let Some(ref v) = dto.planned_at {
        updates.push("planned_at = ?".into());
        param_values.push(Box::new(v.clone()));
    }

    param_values.push(Box::new(dto.id.clone()));

    let sql = format!("UPDATE trade_plan SET {} WHERE id = ?", updates.join(", "));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        param_values.iter().map(|p| p.as_ref()).collect();

    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::Database(format!("更新交易计划失败: {}", e)))?;

    find_by_id(conn, &dto.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let _existing = find_by_id(conn, id)?;
    conn.execute("DELETE FROM trade_plan WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(format!("删除交易计划失败: {}", e)))?;
    Ok(())
}
