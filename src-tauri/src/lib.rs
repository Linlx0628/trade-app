//! TradeApp - 期货/股票交易计划与复盘系统
//!
//! Tauri V2 应用后端入口

mod commands;
mod db;
mod error;
mod models;
mod services;

use tauri::Manager;
use tauri_plugin_store::StoreExt;

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

            // 自动备份：启动时执行
            let app_handle = app.handle().clone();
            let app_dir = app.path().app_data_dir()
                .expect("无法获取应用数据目录");

            let backup_config: models::backup::BackupConfig = {
                let store = app.store("backup.json")
                    .expect("无法打开备份存储");
                store.get("auto_backup_config")
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default()
            };

            if backup_config.enabled && backup_config.backup_on_start {
                let state = app_handle.state::<db::DbState>();
                if let Ok(_info) = services::AutoBackupService::perform_backup(
                    &state, &backup_config, &app_dir,
                ) {
                    log::info!("启动时自动备份完成");
                }
            }

            // 定时自动备份
            if backup_config.enabled && backup_config.interval_minutes > 0 {
                let interval = std::time::Duration::from_secs(backup_config.interval_minutes * 60);
                let handle = app_handle.clone();
                let config = backup_config.clone();
                let dir = app_dir.clone();
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(interval);
                        let state = handle.state::<db::DbState>();
                        if services::AutoBackupService::perform_backup(
                            &state, &config, &dir,
                        ).is_ok() {
                            log::info!("定时自动备份完成");
                        }
                    }
                });
            }

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
            // 交易模板
            commands::trade_template::get_trade_templates,
            commands::trade_template::get_trade_template,
            commands::trade_template::create_trade_template,
            commands::trade_template::update_trade_template,
            commands::trade_template::delete_trade_template,
            commands::trade_template::create_template_from_plan,
            commands::trade_template::create_plan_from_template,
            // 数据导入导出
            commands::data_io::export_trade_logs,
            commands::data_io::export_trade_plans,
            commands::data_io::preview_import,
            commands::data_io::import_trade_logs,
            commands::data_io::create_backup,
            // 仪表盘
            commands::dashboard::get_dashboard_stats,
            commands::dashboard::get_pnl_trend,
            commands::dashboard::get_symbol_pnl,
            // 搜索
            commands::search::global_search,
            // 自动备份
            commands::backup::get_backup_config,
            commands::backup::update_backup_config,
            commands::backup::perform_backup,
            commands::backup::list_backups,
            commands::backup::restore_backup,
            commands::backup::delete_backup,
            commands::backup::get_backup_status,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
