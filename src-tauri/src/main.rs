// 防止 Windows 发布模式下出现额外的控制台窗口，请勿删除！！
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    trade_app_lib::run()
}
