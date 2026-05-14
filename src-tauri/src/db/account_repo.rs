//! 账户数据访问层
//!
//! 提供账户表的 CRUD 操作

use crate::error::AppError;
use crate::models::{Account, CreateAccountDto, UpdateAccountDto};
use rusqlite::{params, Connection};

/// 查询所有账户
///
/// 返回按创建时间降序排列的所有账户列表
pub fn find_all(conn: &Connection) -> Result<Vec<Account>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, balance, risk_ratio, point_value, market_type, description, is_active, created_at, updated_at
         FROM account
         ORDER BY created_at DESC",
    )?;

    let accounts = stmt
        .query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                balance: row.get(2)?,
                risk_ratio: row.get(3)?,
                point_value: row.get(4)?,
                market_type: row.get(5)?,
                description: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(format!("查询账户列表失败: {}", e)))?;

    Ok(accounts)
}

/// 根据 ID 查询账户
pub fn find_by_id(conn: &Connection, id: &str) -> Result<Account, AppError> {
    conn.query_row(
        "SELECT id, name, balance, risk_ratio, point_value, market_type, description, is_active, created_at, updated_at
         FROM account
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                balance: row.get(2)?,
                risk_ratio: row.get(3)?,
                point_value: row.get(4)?,
                market_type: row.get(5)?,
                description: row.get(6)?,
                is_active: row.get::<_, i32>(7)? == 1,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("账户不存在: {}", id))
        }
        _ => AppError::Database(format!("查询账户失败: {}", e)),
    })
}

/// 创建新账户
pub fn create(conn: &Connection, dto: &CreateAccountDto) -> Result<Account, AppError> {
    // 验证账户名称
    if dto.name.trim().is_empty() {
        return Err(AppError::Validation("账户名称不能为空".to_string()));
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let is_active = 1_i32; // 默认激活

    conn.execute(
        "INSERT INTO account (id, name, balance, risk_ratio, point_value, market_type, description, is_active, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            dto.name,
            dto.balance,
            dto.risk_ratio,
            dto.point_value,
            dto.market_type,
            dto.description,
            is_active,
            now,
            now,
        ],
    )
    .map_err(|e| AppError::Database(format!("创建账户失败: {}", e)))?;

    // 查询并返回新创建的账户
    find_by_id(conn, &id)
}

/// 更新账户信息
///
/// 只更新 DTO 中提供的非空字段
pub fn update(conn: &Connection, dto: &UpdateAccountDto) -> Result<Account, AppError> {
    // 先确认账户存在
    let _existing = find_by_id(conn, &dto.id)?;

    let now = chrono::Utc::now().to_rfc3339();

    // 构建动态更新 SQL
    let mut updates: Vec<String> = vec!["updated_at = ?".to_string()];
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

    if let Some(ref name) = dto.name {
        if name.trim().is_empty() {
            return Err(AppError::Validation("账户名称不能为空".to_string()));
        }
        updates.push("name = ?".to_string());
        param_values.push(Box::new(name.clone()));
    }

    if let Some(balance) = dto.balance {
        updates.push("balance = ?".to_string());
        param_values.push(Box::new(balance));
    }

    if let Some(risk_ratio) = dto.risk_ratio {
        updates.push("risk_ratio = ?".to_string());
        param_values.push(Box::new(risk_ratio));
    }

    if let Some(point_value) = dto.point_value {
        updates.push("point_value = ?".to_string());
        param_values.push(Box::new(point_value));
    }

    if let Some(ref market_type) = dto.market_type {
        updates.push("market_type = ?".to_string());
        param_values.push(Box::new(market_type.clone()));
    }

    if let Some(ref description) = dto.description {
        updates.push("description = ?".to_string());
        param_values.push(Box::new(description.clone()));
    }

    if let Some(is_active) = dto.is_active {
        updates.push("is_active = ?".to_string());
        param_values.push(Box::new(if is_active { 1_i32 } else { 0_i32 }));
    }

    // 添加 WHERE 条件的参数
    param_values.push(Box::new(dto.id.clone()));

    let sql = format!(
        "UPDATE account SET {} WHERE id = ?",
        updates.join(", ")
    );

    // 构建参数引用
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
        .iter()
        .map(|p| p.as_ref())
        .collect();

    conn.execute(&sql, param_refs.as_slice())
        .map_err(|e| AppError::Database(format!("更新账户失败: {}", e)))?;

    // 查询并返回更新后的账户
    find_by_id(conn, &dto.id)
}

/// 删除账户
///
/// 关联的交易计划和日志会通过外键级联删除
pub fn delete(conn: &Connection, id: &str) -> Result<(), AppError> {
    // 先确认账户存在
    let _existing = find_by_id(conn, id)?;

    conn.execute("DELETE FROM account WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(format!("删除账户失败: {}", e)))?;

    Ok(())
}

/// 查询账户统计数据
///
/// 聚合该账户下所有已平仓交易的统计信息
pub fn find_stats(conn: &Connection, account_id: &str) -> Result<crate::models::AccountStats, AppError> {
    // 先确认账户存在
    let account = find_by_id(conn, account_id)?;

    // 聚合交易统计
    let stats_row = conn.query_row(
        "SELECT
            COUNT(*) as total_trades,
            COALESCE(SUM(CASE WHEN pnl > 0 THEN 1 ELSE 0 END), 0) as win_trades,
            COALESCE(SUM(CASE WHEN pnl <= 0 THEN 1 ELSE 0 END), 0) as loss_trades,
            COALESCE(SUM(pnl), 0.0) as total_pnl,
            COALESCE(SUM(commission), 0.0) as total_commission,
            COALESCE(SUM(pnl) - SUM(commission), 0.0) as net_pnl,
            COALESCE(AVG(CASE WHEN pnl > 0 THEN pnl END), 0.0) as avg_profit,
            COALESCE(AVG(CASE WHEN pnl <= 0 THEN pnl END), 0.0) as avg_loss,
            COALESCE(MAX(CASE WHEN pnl > 0 THEN pnl END), 0.0) as max_profit,
            COALESCE(MIN(CASE WHEN pnl <= 0 THEN pnl END), 0.0) as max_loss
         FROM trade_log
         WHERE account_id = ?1 AND status = 'closed'",
        params![account_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,   // total_trades
                row.get::<_, i64>(1)?,   // win_trades
                row.get::<_, i64>(2)?,   // loss_trades
                row.get::<_, f64>(3)?,   // total_pnl
                row.get::<_, f64>(4)?,   // total_commission
                row.get::<_, f64>(5)?,   // net_pnl
                row.get::<_, f64>(6)?,   // avg_profit
                row.get::<_, f64>(7)?,   // avg_loss
                row.get::<_, f64>(8)?,   // max_profit
                row.get::<_, f64>(9)?,   // max_loss
            ))
        },
    )
    .unwrap_or((0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));

    let (total_trades, win_trades, loss_trades, total_pnl, total_commission, net_pnl, avg_profit, avg_loss, max_profit, max_loss) = stats_row;

    // 计算胜率
    let win_rate = if total_trades > 0 {
        win_trades as f64 / total_trades as f64
    } else {
        0.0
    };

    // 计算盈亏比（Profit Factor）
    let total_wins = avg_profit * win_trades as f64;
    let total_losses = avg_loss.abs() * loss_trades as f64;
    let profit_factor = if total_losses > 0.0 {
        total_wins / total_losses
    } else if total_wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    // 净值 = 余额 + 净盈亏
    let net_value = account.balance + net_pnl;

    Ok(crate::models::AccountStats {
        account_id: account.id.clone(),
        account_name: account.name,
        balance: account.balance,
        net_value,
        total_pnl,
        total_commission,
        net_pnl,
        total_trades,
        win_trades,
        loss_trades,
        win_rate,
        avg_profit,
        avg_loss,
        profit_factor,
        max_profit,
        max_loss,
    })
}
