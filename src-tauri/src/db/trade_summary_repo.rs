use crate::error::AppError;
use crate::models::trade_summary::{CreateTradeSummaryDto, TradeSummary, UpdateTradeSummaryDto};
use rusqlite::{params, Connection};

fn map_row(row: &rusqlite::Row<'_>) -> Result<TradeSummary, rusqlite::Error> {
    Ok(TradeSummary {
        id: row.get(0)?,
        account_id: row.get(1)?,
        summary_type: row.get(2)?,
        summary_date: row.get(3)?,
        total_trades: row.get(4)?,
        win_trades: row.get(5)?,
        loss_trades: row.get(6)?,
        total_pnl: row.get(7)?,
        total_commission: row.get(8)?,
        net_pnl: row.get(9)?,
        win_rate: row.get(10)?,
        avg_profit: row.get(11)?,
        avg_loss: row.get(12)?,
        profit_factor: row.get(13)?,
        max_profit: row.get(14)?,
        max_loss: row.get(15)?,
        emotion_score: row.get(16)?,
        market_view: row.get(17)?,
        lessons: row.get(18)?,
        improvement: row.get(19)?,
        tags: row.get(20)?,
        created_at: row.get(21)?,
        updated_at: row.get(22)?,
    })
}

const COLS: &str = "id, account_id, summary_type, summary_date, total_trades, win_trades, loss_trades, total_pnl, total_commission, net_pnl, win_rate, avg_profit, avg_loss, profit_factor, max_profit, max_loss, emotion_score, market_view, lessons, improvement, tags, created_at, updated_at";

pub fn find_by_account(
    conn: &Connection,
    account_id: &str,
    summary_type: Option<&str>,
) -> Result<Vec<TradeSummary>, AppError> {
    let (sql, p): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match summary_type {
        Some(t) => (
            format!("SELECT {} FROM trade_summary WHERE account_id = ?1 AND summary_type = ?2 ORDER BY summary_date DESC", COLS),
            vec![Box::new(account_id.to_string()), Box::new(t.to_string())],
        ),
        None => (
            format!("SELECT {} FROM trade_summary WHERE account_id = ?1 ORDER BY summary_date DESC", COLS),
            vec![Box::new(account_id.to_string())],
        ),
    };
    let mut stmt = conn.prepare(&sql)?;
    let refs: Vec<&dyn rusqlite::types::ToSql> = p.iter().map(|x| x.as_ref()).collect();
    let list = stmt.query_map(refs.as_slice(), |r| map_row(r))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(format!("查询交易总结失败: {}", e)))?;
    Ok(list)
}

pub fn find_by_id(conn: &Connection, id: &str) -> Result<TradeSummary, AppError> {
    let sql = format!("SELECT {} FROM trade_summary WHERE id = ?1", COLS);
    conn.query_row(&sql, params![id], |r| map_row(r))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("交易总结不存在: {}", id)),
            _ => AppError::Database(format!("查询交易总结失败: {}", e)),
        })
}

/// 自动聚合统计数据并创建总结
pub fn create_with_aggregation(
    conn: &Connection,
    dto: &CreateTradeSummaryDto,
) -> Result<TradeSummary, AppError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&dto.tags).map_err(AppError::Serialization)?;

    // 聚合该日期范围内的交易日志
    let stats = aggregate_stats(conn, &dto.account_id, &dto.summary_date, &dto.summary_type)?;

    conn.execute(
        "INSERT INTO trade_summary (id, account_id, summary_type, summary_date, total_trades, win_trades, loss_trades, total_pnl, total_commission, net_pnl, win_rate, avg_profit, avg_loss, profit_factor, max_profit, max_loss, emotion_score, market_view, lessons, improvement, tags, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23)",
        params![id, dto.account_id, dto.summary_type, dto.summary_date,
            stats.total_trades, stats.win_trades, stats.loss_trades,
            stats.total_pnl, stats.total_commission, stats.net_pnl,
            stats.win_rate, stats.avg_profit, stats.avg_loss,
            stats.profit_factor, stats.max_profit, stats.max_loss,
            dto.emotion_score, dto.market_view, dto.lessons, dto.improvement, tags_json, now, now],
    ).map_err(|e| AppError::Database(format!("创建交易总结失败: {}", e)))?;

    find_by_id(conn, &id)
}

pub fn update(conn: &Connection, dto: &UpdateTradeSummaryDto) -> Result<TradeSummary, AppError> {
    let _ = find_by_id(conn, &dto.id)?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut sets: Vec<String> = vec!["updated_at = ?".into()];
    let mut vals: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    if let Some(v) = dto.emotion_score { sets.push("emotion_score = ?".into()); vals.push(Box::new(v)); }
    if let Some(ref v) = dto.market_view { sets.push("market_view = ?".into()); vals.push(Box::new(v.clone())); }
    if let Some(ref v) = dto.lessons { sets.push("lessons = ?".into()); vals.push(Box::new(v.clone())); }
    if let Some(ref v) = dto.improvement { sets.push("improvement = ?".into()); vals.push(Box::new(v.clone())); }
    if let Some(ref v) = dto.tags {
        let json = serde_json::to_string(v).map_err(AppError::Serialization)?;
        sets.push("tags = ?".into()); vals.push(Box::new(json));
    }

    vals.push(Box::new(dto.id.clone()));
    let sql = format!("UPDATE trade_summary SET {} WHERE id = ?", sets.join(", "));
    let refs: Vec<&dyn rusqlite::types::ToSql> = vals.iter().map(|x| x.as_ref()).collect();
    conn.execute(&sql, refs.as_slice()).map_err(|e| AppError::Database(format!("更新交易总结失败: {}", e)))?;
    find_by_id(conn, &dto.id)
}

pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    let _ = find_by_id(conn, id)?;
    conn.execute("DELETE FROM trade_summary WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(format!("删除交易总结失败: {}", e)))?;
    Ok(())
}

struct AggStats {
    total_trades: i32, win_trades: i32, loss_trades: i32,
    total_pnl: f64, total_commission: f64, net_pnl: f64,
    win_rate: f64, avg_profit: f64, avg_loss: f64,
    profit_factor: f64, max_profit: f64, max_loss: f64,
}

fn aggregate_stats(
    conn: &Connection,
    account_id: &str,
    summary_date: &str,
    summary_type: &str,
) -> Result<AggStats, AppError> {
    // 简单按日期匹配聚合（当天/当周/当月的 closed 交易）
    let date_filter = match summary_type {
        "weekly" => format!("substr(exit_time, 1, 10) >= date('{}', '-6 days') AND substr(exit_time, 1, 10) <= '{}'", summary_date, summary_date),
        "monthly" => format!("substr(exit_time, 1, 7) = substr('{}', 1, 7)", summary_date),
        _ => format!("substr(exit_time, 1, 10) = '{}'", summary_date),
    };

    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END),0), COALESCE(SUM(CASE WHEN pnl <= 0 THEN 1 ELSE 0 END),0), COALESCE(SUM(pnl),0), COALESCE(SUM(commission),0), COALESCE(SUM(pnl)-SUM(commission),0), COALESCE(AVG(CASE WHEN pnl > 0 THEN pnl END),0), COALESCE(AVG(CASE WHEN pnl <= 0 THEN pnl END),0), COALESCE(MAX(CASE WHEN pnl > 0 THEN pnl END),0), COALESCE(MIN(CASE WHEN pnl <= 0 THEN pnl END),0) FROM trade_log WHERE account_id = ?1 AND status = 'closed' AND {}",
        date_filter
    );

    let row = conn.query_row(&sql, params![account_id], |r| {
        Ok((
            r.get::<_, i64>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?,
            r.get::<_, f64>(3)?, r.get::<_, f64>(4)?, r.get::<_, f64>(5)?,
            r.get::<_, f64>(6)?, r.get::<_, f64>(7)?, r.get::<_, f64>(8)?, r.get::<_, f64>(9)?,
        ))
    }).unwrap_or((0,0,0,0.0,0.0,0.0,0.0,0.0,0.0,0.0));

    let (total, wins, losses, total_pnl, commission, net_pnl, avg_profit, avg_loss, max_profit, max_loss) = row;
    let win_rate = if total > 0 { wins as f64 / total as f64 } else { 0.0 };
    let total_win = avg_profit * wins as f64;
    let total_lose = avg_loss.abs() * losses as f64;
    let profit_factor = if total_lose > 0.0 { total_win / total_lose } else if total_win > 0.0 { f64::INFINITY } else { 0.0 };

    Ok(AggStats {
        total_trades: total as i32, win_trades: wins as i32, loss_trades: losses as i32,
        total_pnl, total_commission: commission, net_pnl, win_rate, avg_profit, avg_loss,
        profit_factor: if profit_factor.is_infinite() { 0.0 } else { profit_factor },
        max_profit, max_loss,
    })
}
