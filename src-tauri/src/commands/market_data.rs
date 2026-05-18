use crate::error::AppError;
use crate::models::market_data::{KlineRequest, MarketQuote, SymbolInfo};
use crate::services::market_data_service::MarketDataService;

#[tauri::command]
pub async fn get_quote(symbol: String) -> Result<MarketQuote, AppError> {
    MarketDataService::get_quote(&symbol).await
}

#[tauri::command]
pub async fn get_quotes(symbols: Vec<String>) -> Result<Vec<MarketQuote>, AppError> {
    MarketDataService::get_quotes(&symbols).await
}

#[tauri::command]
pub async fn get_kline_data(request: KlineRequest) -> Result<Vec<crate::models::market_data::KlineData>, AppError> {
    let count = request.count.unwrap_or(100);
    MarketDataService::get_kline(&request.symbol, &request.period, count).await
}

#[tauri::command]
pub async fn subscribe_market(symbols: Vec<String>, interval_ms: Option<u64>) -> Result<(), AppError> {
    let interval = interval_ms.unwrap_or(3000);
    MarketDataService::start_subscription(symbols, interval);
    Ok(())
}

#[tauri::command]
pub async fn unsubscribe_market() -> Result<(), AppError> {
    MarketDataService::stop_subscription();
    Ok(())
}

#[tauri::command]
pub fn search_symbol(keyword: String) -> Result<Vec<SymbolInfo>, AppError> {
    Ok(MarketDataService::search_symbol(&keyword))
}
