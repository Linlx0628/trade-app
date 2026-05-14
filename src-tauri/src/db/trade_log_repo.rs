use crate::error::AppError;
use crate::models::trade_log::{CreateTradeLogDto, TradeLog, UpdateTradeLogDto};
use rusqlite::{params, Connection};

fn map_row(row: &rusqlite::Row<'_>) -> Result<TradeLog, rusqlite::Error> {
    Ok(TradeLog {
        id: row.get(0)?,
        account_id: row.get(1)?,
        plan_id: row.get(2)?,
        symbol: row.get(3)?,
        name: row.get(4)?,
        direction: row.get(5)?,
        entry_price: row.get(6)?,
        exit_price: row.get(7)?,
        stop_loss: row.get(8)?,
        lots: row.get(9)?,
        pnl: row.get(10)?,
        pnl_points: row.get(11)?,
        commission: row.get(12)?,
        market_type: row.get(13)?,
        status: row.get(14)?,
        entry_time: row.get(15)?,
        exit_time: row.get(16)?,
        tags: row.get(17)?,
        notes: row.get(18)?,
        images: row.get(19)?,
        ai_feedback: row.get(20)?,
        emotion_before: row.get(21)?,
        emotion_after: row.get(22)?,
        confidence: row.get(23)?,
        created_at: row.get(24)?,
        updated_at: row.get(25)?,
    })
}

const COLS: &str = "id, account_id, plan_id, symbol, name, direction, entry_price, exit_price, stop_loss, lots, pnl, pnl_points, commission, market_type, status, entry_time, exit_time, tags, notes, images, ai_feedback, emotion_before, emotion_after, confidence, created_at, updated_at";

pub fn find_by_account(
    conn: &Connection,
    account_id: &str,
    status: Option<&str>,
) -> Result<Vec<TradeLog>, AppError> {
    let (sql, p): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match status {
        Some(s) => (
            format!("SELECT {} FROM trade_log WHERE account_id = ?1 AND status = ?2 ORDER BY entry_time DESC NULLS LAST, created_at DESC", COLS),
            vec![Box::new(account_id.to_string()), Box::new(s.to_string())],
        ),
        None => (
            format!("SELECT {} FROM trade_log WHERE account_id = ?1 ORDER BY entry_time DESC NULLS LAST, created_at DESC", COLS),
            vec![Box::new(account_id.to_string())],
        ),
    };
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = p.iter().map(|x| x.as_ref()).collect();
    let logs = stmt.query_map(refs.as_slice(), |r| map_row(r))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(format!("查询交易日志失败: {}", e)))?;
    Ok(logs)
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<TradeLog, AppError> {
    let sql = format!("SELECT {} FROM trade_log WHERE id = ?1", COLS);
    conn.query_row(&sql, params![id], |r| map_row(r))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("交易日志不存在: {}", id)),
            _ => AppError::Database(format!("查询交易日志失败: {}", e)),
        })
}

pub fn create(conn: &Connection, dto: &CreateTradeLogDto) -> Result<TradeLog, AppError> {
    if dto.symbol.trim().is_empty() {
        return Err(AppError::Validation("交易品种不能为空".to_string()));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&dto.tags).map_err(AppError::Serialization)?;
    let plan_id = dto.plan_id.as_deref();

    conn.execute(
        "INSERT INTO trade_log (id, account_id, plan_id, symbol, name, direction, entry_price, exit_price, stop_loss, lots, pnl, pnl_points, commission, market_type, status, entry_time, exit_time, tags, notes, images, ai_feedback, emotion_before, emotion_after, confidence, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,'[]','',?20,'',?21,?22,?23)",
        params![id, dto.account_id, plan_id, dto.symbol, dto.name, dto.direction, dto.entry_price, dto.exit_price, dto.stop_loss, dto.lots, dto.pnl, dto.pnl_points, dto.commission, dto.market_type, dto.status, dto.entry_time, dto.exit_time, tags_json, dto.notes, dto.emotion_before, dto.confidence, now, now],
    ).map_err(|e| AppError::Database(format!("创建交易日志失败: {}", e)))?;

    find_by_id(conn, &id)
}

pub fn update(conn: &Connection, dto: &UpdateTradeLogDto) -> Result<TradeLog, AppError> {
    let _ = find_by_id(conn, &dto.id)?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut sets: Vec<String> = vec!["updated_at = ?".into()];
    let mut vals: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    macro_rules! opt {
        ($field:expr, $col:expr) => {
            if let Some(ref v) = $field { sets.push(format!("{} = ?", $col)); vals.push(Box::new(v.clone())); }
        };
        ($field:expr, $col:expr, num) => {
            if let Some(v) = $field { sets.push(format!("{} = ?", $col)); vals.push(Box::new(v)); }
        };
    }
    opt!(dto.symbol, "symbol");
    opt!(dto.name, "name");
    opt!(dto.direction, "direction");
    opt!(dto.entry_price, "entry_price", num);
    opt!(dto.exit_price, "exit_price", num);
    opt!(dto.stop_loss, "stop_loss", num);
    opt!(dto.lots, "lots", num);
    opt!(dto.pnl, "pnl", num);
    opt!(dto.pnl_points, "pnl_points", num);
    opt!(dto.commission, "commission", num);
    opt!(dto.status, "status");
    opt!(dto.exit_time, "exit_time");
    opt!(dto.emotion_after, "emotion_after");
    if let Some(v) = dto.confidence { sets.push("confidence = ?".into()); vals.push(Box::new(v)); }
    if let Some(ref v) = dto.tags {
        let json = serde_json::to_string(v).map_err(AppError::Serialization)?;
        sets.push("tags = ?".into()); vals.push(Box::new(json));
    }
    opt!(dto.notes, "notes");

    vals.push(Box::new(dto.id.clone()));
    let sql = format!("UPDATE trade_log SET {} WHERE id = ?", sets.join(", "));
    let refs: Vec<&dyn rusqlite::types::ToSql> = vals.iter().map(|x| x.as_ref()).collect();
    conn.execute(&sql, refs.as_slice()).map_err(|e| AppError::Database(format!("更新交易日志失败: {}", e)))?;
    find_by_id(conn, &dto.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let _ = find_by_id(conn, id)?;
    conn.execute("DELETE FROM trade_log WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(format!("删除交易日志失败: {}", e)))?;
    Ok(())
}
