//! TradeApp - 期货/股票交易计划与复盘系统
//!
//! Tauri V2 应用后端入口

mod commands;
mod db;
mod error;
mod models;
mod services;

use tauri::Manager;

/// 应用初始化和启动
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 注册 Tauri 插件
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        // 初始化数据库（在 setup 钩子中执行）
        .setup(|app| {
            // 初始化日志
            env_logger::init();

            // 初始化数据库连接并运行迁移
            let db_state = db::init_db(app)?;
            app.manage(db_state);

            log::info!("TradeApp 后端初始化完成");
            Ok(())
        })
        // 注册 Tauri Commands
        .invoke_handler(tauri::generate_handler![
            // 账户管理
            commands::account::get_accounts,
            commands::account::get_account,
            commands::account::create_account,
            commands::account::update_account,
            commands::account::delete_account,
            commands::account::get_account_stats,
            commands::account::calculate_trade,
            // 交易计划
            commands::trade_plan::get_trade_plans,
            commands::trade_plan::get_trade_plan,
            commands::trade_plan::create_trade_plan,
            commands::trade_plan::update_trade_plan,
            commands::trade_plan::delete_trade_plan,
            commands::trade_plan::execute_trade_plan,
            // 交易日志
            commands::trade_log::get_trade_logs,
            commands::trade_log::get_trade_log,
            commands::trade_log::create_trade_log,
            commands::trade_log::update_trade_log,
            commands::trade_log::delete_trade_log,
            // 交易总结
            commands::trade_summary::get_trade_summaries,
            commands::trade_summary::get_trade_summary,
            commands::trade_summary::create_trade_summary,
            commands::trade_summary::update_trade_summary,
            commands::trade_summary::delete_trade_summary,
            // AI
            commands::ai::ai_chat,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
