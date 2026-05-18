use crate::error::AppError;
use crate::models::market_data::{KlineData, MarketQuote, SymbolInfo};
use std::collections::HashMap;
use std::sync::Mutex;

static SUBSCRIPTION: Mutex<Option<SubscriptionState>> = Mutex::new(None);

struct SubscriptionState {
    pub symbols: Vec<String>,
    pub interval_ms: u64,
}

pub struct MarketDataService;

impl MarketDataService {
    pub async fn get_quote(symbol: &str) -> Result<MarketQuote, AppError> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| AppError::Database(format!("HTTP 客户端创建失败: {}", e)))?;

        let url = format!("https://hq.sinajs.cn/list={}", symbol);
        let resp = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取行情响应失败: {}", e)))?;

        Self::parse_sina_quote(symbol, &text)
    }

    pub async fn get_quotes(symbols: &[String]) -> Result<Vec<MarketQuote>, AppError> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| AppError::Database(format!("HTTP 客户端创建失败: {}", e)))?;

        let symbols_str = symbols.join(",");
        let url = format!("https://hq.sinajs.cn/list={}", symbols_str);
        let resp = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取行情响应失败: {}", e)))?;

        let mut quotes = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("var hq_str_") {
                if let Some(quote) = Self::parse_sina_line(line) {
                    quotes.push(quote);
                }
            }
        }

        Ok(quotes)
    }

    pub async fn get_kline(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| AppError::Database(format!("HTTP 客户端创建失败: {}", e)))?;

        let sina_period = match period {
            "1m" => "5",
            "5m" => "15",
            "15m" => "30",
            "30m" => "60",
            "60m" => "120",
            "day" | "daily" => "1440",
            _ => "1440",
        };

        let is_futures = !symbol.starts_with("sh") && !symbol.starts_with("sz")
            && !symbol.starts_with("s_sh") && !symbol.starts_with("s_sz");

        let url = if is_futures {
            format!(
                "https://finance.sina.com.cn/futures/api/jsonp.php/IO.XSRV2.CallbackList/xxx/CfuturesApiService.getKLineData?symbol={}&scale={}&datalen={}",
                symbol, sina_period, count
            )
        } else {
            let pure_code = symbol
                .trim_start_matches("sh")
                .trim_start_matches("sz")
                .trim_start_matches("s_sh")
                .trim_start_matches("s_sz");
            format!(
                "https://money.finance.sina.com.cn/quotes_service/api/json_v2.php/CN_MarketData.getKLineData?symbol={}&scale={}&ma=no&datalen={}",
                pure_code, sina_period, count
            )
        };

        let resp = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("K线请求失败: {}", e)))?;

        let text = resp
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取K线响应失败: {}", e)))?;

        Self::parse_kline_response(&text)
    }

    pub fn search_symbol(keyword: &str) -> Vec<SymbolInfo> {
        let futures_map: &[(&str, &str)] = &[
            ("rb", "螺纹钢"), ("hc", "热卷"), ("i", "铁矿石"),
            ("j", "焦炭"), ("jm", "焦煤"), ("ZC", "动力煤"),
            ("cu", "沪铜"), ("al", "沪铝"), ("zn", "沪锌"),
            ("pb", "沪铅"), ("ni", "沪镍"), ("sn", "沪锡"),
            ("au", "沪金"), ("ag", "沪银"),
            ("IF", "沪深300"), ("IC", "中证500"), ("IH", "上证50"), ("IM", "中证1000"),
            ("m", "豆粕"), ("y", "豆油"), ("a", "豆一"), ("b", "豆二"),
            ("p", "棕榈油"), ("c", "玉米"), ("cs", "淀粉"),
            ("CF", "棉花"), ("SR", "白糖"), ("TA", "PTA"), ("MA", "甲醇"),
            ("OI", "菜油"), ("FG", "玻璃"), ("SA", "纯碱"), ("EG", "乙二醇"),
            ("ap", "苹果"), ("CJ", "红枣"), ("UR", "尿素"),
            ("T", "十年国债"), ("TF", "五年国债"), ("TS", "两年国债"),
        ];

        let stock_map: &[(&str, &str)] = &[
            ("600519", "贵州茅台"), ("000858", "五粮液"),
            ("601318", "中国平安"), ("600036", "招商银行"),
            ("000333", "美的集团"), ("002594", "比亚迪"),
            ("601012", "隆基绿能"), ("300750", "宁德时代"),
            ("600900", "长江电力"), ("601899", "紫金矿业"),
        ];

        let mut results = Vec::new();

        for &(code, name) in futures_map {
            if code.to_lowercase().contains(&keyword.to_lowercase())
                || name.contains(keyword)
            {
                results.push(SymbolInfo {
                    symbol: code.to_string(),
                    name: name.to_string(),
                    market_type: "futures".to_string(),
                });
            }
        }

        for &(code, name) in stock_map {
            if code.contains(keyword) || name.contains(keyword) {
                results.push(SymbolInfo {
                    symbol: format!("sh{}", code),
                    name: name.to_string(),
                    market_type: "stock".to_string(),
                });
            }
        }

        results
    }

    pub fn start_subscription(symbols: Vec<String>, interval_ms: u64) {
        if let Ok(mut sub) = SUBSCRIPTION.lock() {
            *sub = Some(SubscriptionState {
                symbols,
                interval_ms,
            });
        }
    }

    pub fn stop_subscription() {
        if let Ok(mut sub) = SUBSCRIPTION.lock() {
            *sub = None;
        }
    }

    pub fn get_subscription_symbols() -> Vec<String> {
        SUBSCRIPTION
            .lock()
            .ok()
            .and_then(|s| s.as_ref().map(|state| state.symbols.clone()))
            .unwrap_or_default()
    }

    fn parse_sina_quote(symbol: &str, text: &str) -> Result<MarketQuote, AppError> {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with(&format!("var hq_str_{}=", symbol)) {
                return Self::parse_sina_line(line).ok_or_else(|| {
                    AppError::Database(format!("解析行情数据失败: {}", symbol))
                });
            }
        }
        Err(AppError::NotFound(format!("未找到行情数据: {}", symbol)))
    }

    fn parse_sina_line(line: &str) -> Option<MarketQuote> {
        let eq_pos = line.find('=')?;
        let symbol = line["var hq_str_".len()..eq_pos].to_string();

        let value_start = eq_pos + 2; // skip '="'
        let value_end = line.rfind('"')?;
        let value = &line[value_start..value_end];
        if value.is_empty() {
            return None;
        }

        let fields: Vec<&str> = value.split(',').collect();
        if fields.len() < 32 {
            return None;
        }

        let name = fields[0].to_string();
        let open = fields[1].parse().unwrap_or(0.0);
        let prev_close = fields[2].parse().unwrap_or(0.0);
        let current = fields[3].parse().unwrap_or(0.0);
        let high = fields[4].parse().unwrap_or(0.0);
        let low = fields[5].parse().unwrap_or(0.0);
        let bid = fields[6].parse().unwrap_or(0.0);
        let ask = fields[7].parse().unwrap_or(0.0);
        let volume = fields[8].parse().unwrap_or(0.0);
        let amount = fields[9].parse().unwrap_or(0.0);

        let change = if prev_close != 0.0 {
            current - prev_close
        } else {
            0.0
        };
        let change_pct = if prev_close != 0.0 {
            (current - prev_close) / prev_close * 100.0
        } else {
            0.0
        };

        let timestamp = if fields.len() > 31 {
            format!("{} {}", fields[30], fields[31])
        } else {
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        };

        Some(MarketQuote {
            symbol,
            name,
            open,
            prev_close,
            current,
            high,
            low,
            bid,
            ask,
            volume,
            amount,
            change,
            change_pct,
            timestamp,
        })
    }

    fn parse_kline_response(text: &str) -> Result<Vec<KlineData>, AppError> {
        let cleaned = text
            .trim()
            .trim_start_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '(')
            .trim_end_matches(|c: char| c == ')' || c == ';' || c == '\n');

        let stripped = cleaned
            .trim_start_matches(|c: char| c.is_alphabetic() || c == '.' || c == '(')
            .trim_end_matches(");")
            .trim_start_matches('(');

        let items: Vec<HashMap<String, serde_json::Value>> =
            if let Ok(parsed) = serde_json::from_str(stripped) {
                parsed
            } else if let Ok(parsed) = serde_json::from_str(cleaned) {
                parsed
            } else {
                return Ok(Vec::new());
            };

        let mut klines = Vec::new();
        for item in &items {
            let get_f = |key: &str| -> f64 {
                item.get(key)
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .or_else(|| item.get(key).and_then(|v| v.as_f64()))
                    .unwrap_or(0.0)
            };

            let timestamp = item
                .get("day")
                .or_else(|| item.get("d"))
                .or_else(|| item.get("date"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            klines.push(KlineData {
                timestamp,
                open: get_f("open"),
                high: get_f("high"),
                low: get_f("low"),
                close: get_f("close"),
                volume: get_f("volume"),
            });
        }

        Ok(klines)
    }
}
