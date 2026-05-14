//! 数据库管理模块
//!
//! 负责数据库连接初始化、迁移管理

pub mod account_repo;
pub mod trade_log_repo;
pub mod trade_plan_repo;
pub mod trade_summary_repo;
pub mod trade_template_repo;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

/// 数据库连接状态，通过 Mutex 保证线程安全
pub struct DbState {
    pub conn: Mutex<Connection>,
}

/// 初始化数据库连接并运行迁移
///
/// 数据库文件存储在 Tauri 应用数据目录下
pub fn init_db(app: &mut tauri::App) -> Result<DbState, crate::error::AppError> {
    // 获取应用数据目录
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Database(format!("无法获取应用数据目录: {}", e)))?;

    // 确保目录存在
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| crate::error::AppError::Database(format!("无法创建数据目录: {}", e)))?;

    let db_path = app_dir.join("trademind.db");

    // 打开数据库连接
    let conn = Connection::open(&db_path)
        .map_err(|e| crate::error::AppError::Database(format!("无法打开数据库: {}", e)))?;

    // 启用外键约束
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|e| crate::error::AppError::Database(format!("无法启用外键约束: {}", e)))?;

    // 启用 WAL 模式提升并发性能
    conn.execute_batch("PRAGMA journal_mode = WAL;")
        .map_err(|e| crate::error::AppError::Database(format!("无法设置 WAL 模式: {}", e)))?;

    // 运行数据库迁移
    run_migrations(&conn)?;

    Ok(DbState {
        conn: Mutex::new(conn),
    })
}

/// 运行数据库迁移
///
/// 按版本顺序执行 migrations 目录下的 SQL 脚本。
/// 使用 _migrations 跟踪表记录已执行的迁移。
fn run_migrations(conn: &Connection) -> Result<(), crate::error::AppError> {
    // 创建迁移跟踪表
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )
    .map_err(|e| crate::error::AppError::Database(format!("无法创建迁移跟踪表: {}", e)))?;

    // 定义所有迁移（按版本号排列）
    let migrations: Vec<(i64, &str, &str)> = vec![
        (
            1,
            "001_init",
            include_str!("../../migrations/001_init.sql"),
        ),
        (
            2,
            "002_add_templates",
            include_str!("../../migrations/002_add_templates.sql"),
        ),
    ];

    // 逐个执行未应用的迁移
    for (version, name, sql) in &migrations {
        // 检查是否已执行
        let applied: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM _migrations WHERE version = ?1",
                [version],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !applied {
            log::info!("正在执行数据库迁移: {} (版本 {})", name, version);

            // 在事务中执行迁移
            conn.execute_batch("BEGIN TRANSACTION;")?;

            conn.execute_batch(sql).map_err(|e| {
                crate::error::AppError::Database(format!(
                    "迁移 {} 执行失败: {}",
                    name,
                    e
                ))
            })?;

            // 记录已执行的迁移
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO _migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![version, name, now],
            )
            .map_err(|e| {
                crate::error::AppError::Database(format!("无法记录迁移: {}", e))
            })?;

            conn.execute_batch("COMMIT;")?;

            log::info!("数据库迁移 {} 完成", name);
        }
    }

    Ok(())
}
