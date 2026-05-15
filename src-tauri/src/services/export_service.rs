use crate::db::DbState;
use crate::error::AppError;
use std::fs::File;
use std::io::Write;
use tauri::State;
use xlsxwriter::{Workbook, Format};

pub struct ExportService;

impl ExportService {
    pub fn export_trade_logs(
        state: &State<'_, DbState>,
        account_id: &str,
        file_path: &str,
    ) -> Result<String, AppError> {
        let is_xlsx = file_path.to_lowercase().ends_with(".xlsx");
        let data_rows = {
            let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT symbol, name, direction, market_type, entry_price, exit_price, stop_loss, lots, pnl, pnl_points, commission, status, entry_time, exit_time, tags, notes, emotion_before, emotion_after, confidence FROM trade_log WHERE account_id = ?1 ORDER BY entry_time DESC"
            ).map_err(|e| AppError::Database(format!("{}", e)))?;
            let mut rows = stmt.query([account_id]).map_err(|e| AppError::Database(format!("{}", e)))?;
            let mut out: Vec<Vec<String>> = Vec::new();
            while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("{}", e)))? {
                let s = |i: usize| row.get::<_, String>(i).unwrap_or_default();
                let f = |i: usize| row.get::<_, f64>(i).unwrap_or_default();
                let n = |i: usize| row.get::<_, i64>(i).unwrap_or_default();
                out.push(vec![
                    s(0), s(1), s(2), s(3),
                    f(4).to_string(), f(5).to_string(), f(6).to_string(), f(7).to_string(),
                    f(8).to_string(), f(9).to_string(), f(10).to_string(),
                    s(11), s(12), s(13), s(14), s(15), s(16), s(17), n(18).to_string(),
                ]);
            }
            out
        };

        let headers = ["品种代码","品种名称","方向","市场类型","入场价格","出场价格","止损价格","手数","盈亏金额","盈亏点数","手续费","状态","入场时间","出场时间","标签","备注","情绪(前)","情绪(后)","信心指数"];
        if is_xlsx { write_xlsx(file_path, &headers, &data_rows)?; }
        else { write_csv(file_path, &headers, &data_rows)?; }
        Ok(format!("已导出 {} 条交易日志", data_rows.len()))
    }

    pub fn export_trade_plans(
        state: &State<'_, DbState>,
        account_id: &str,
        file_path: &str,
    ) -> Result<String, AppError> {
        let is_xlsx = file_path.to_lowercase().ends_with(".xlsx");
        let data_rows = {
            let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;
            let mut stmt = conn.prepare(
                "SELECT symbol, name, direction, market_type, entry_price, stop_loss, take_profit, lots, status, strategy, tags, notes, planned_at, created_at FROM trade_plan WHERE account_id = ?1 ORDER BY created_at DESC"
            ).map_err(|e| AppError::Database(format!("{}", e)))?;
            let mut rows = stmt.query([account_id]).map_err(|e| AppError::Database(format!("{}", e)))?;
            let mut out: Vec<Vec<String>> = Vec::new();
            while let Some(row) = rows.next().map_err(|e| AppError::Database(format!("{}", e)))? {
                let s = |i: usize| row.get::<_, String>(i).unwrap_or_default();
                let f = |i: usize| row.get::<_, f64>(i).unwrap_or_default();
                out.push(vec![
                    s(0), s(1), s(2), s(3),
                    f(4).to_string(), f(5).to_string(), f(6).to_string(), f(7).to_string(),
                    s(8), s(9), s(10), s(11), s(12), s(13),
                ]);
            }
            out
        };

        let headers = ["品种代码","品种名称","方向","市场类型","入场价格","止损价格","止盈价格","手数","状态","策略","标签","备注","计划时间","创建时间"];
        if is_xlsx { write_xlsx(file_path, &headers, &data_rows)?; }
        else { write_csv(file_path, &headers, &data_rows)?; }
        Ok(format!("已导出 {} 条交易计划", data_rows.len()))
    }

    pub fn create_backup(
        state: &State<'_, DbState>,
        backup_dir: &str,
    ) -> Result<String, AppError> {
        let conn = state.conn.lock().map_err(|e| AppError::Database(format!("数据库锁失败: {}", e)))?;

        let plan_count: i64 = conn.query_row("SELECT COUNT(*) FROM trade_plan", [], |r| r.get(0)).unwrap_or(0);
        let log_count: i64 = conn.query_row("SELECT COUNT(*) FROM trade_log", [], |r| r.get(0)).unwrap_or(0);
        let summary_count: i64 = conn.query_row("SELECT COUNT(*) FROM trade_summary", [], |r| r.get(0)).unwrap_or(0);
        let account_count: i64 = conn.query_row("SELECT COUNT(*) FROM account", [], |r| r.get(0)).unwrap_or(0);

        let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = format!("{}/trademind_backup_{}.db", backup_dir, now_str);

        conn.execute(&format!("VACUUM INTO '{}'", backup_path), [])
            .map_err(|e| AppError::Database(format!("备份失败: {}", e)))?;

        let meta = serde_json::json!({
            "version": "0.5.0",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "account_count": account_count,
            "plan_count": plan_count,
            "log_count": log_count,
            "summary_count": summary_count,
        });
        let meta_path = format!("{}/trademind_backup_{}.json", backup_dir, now_str);
        let mut meta_file = File::create(&meta_path)
            .map_err(|e| AppError::Database(format!("无法创建元信息文件: {}", e)))?;
        meta_file.write_all(serde_json::to_string_pretty(&meta).unwrap().as_bytes()).ok();

        Ok(format!("备份完成: {} 条计划, {} 条日志, {} 条总结", plan_count, log_count, summary_count))
    }
}

fn write_xlsx(path: &str, headers: &[&str], rows: &[Vec<String>]) -> Result<(), AppError> {
    let mut workbook = Workbook::new(path)
        .map_err(|e| AppError::Database(format!("创建Excel文件失败: {}", e)))?;
    let mut sheet = workbook.add_worksheet(None)
        .map_err(|e| AppError::Database(format!("{}", e)))?;

    let mut header_fmt = Format::new();
    header_fmt.set_bold();
    header_fmt.set_text_wrap();
    let mut cell_fmt = Format::new();
    cell_fmt.set_text_wrap();

    for (col, h) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, h, Some(&header_fmt))
            .map_err(|e| AppError::Database(format!("{}", e)))?;
    }

    for (r, row) in rows.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            sheet.write_string((r + 1) as u32, c as u16, val, Some(&cell_fmt))
                .map_err(|e| AppError::Database(format!("{}", e)))?;
        }
    }

    workbook.close().map_err(|e| AppError::Database(format!("保存Excel失败: {}", e)))?;
    Ok(())
}

fn write_csv(path: &str, headers: &[&str], rows: &[Vec<String>]) -> Result<(), AppError> {
    let mut file = File::create(path)
        .map_err(|e| AppError::Database(format!("无法创建文件: {}", e)))?;
    file.write_all(b"\xEF\xBB\xBF").ok();
    writeln!(file, "{}", headers.iter().map(|h| csv_escape(h)).collect::<Vec<_>>().join(",")).ok();
    for row in rows {
        writeln!(file, "{}", row.iter().map(|v| csv_escape(v)).collect::<Vec<_>>().join(",")).ok();
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
