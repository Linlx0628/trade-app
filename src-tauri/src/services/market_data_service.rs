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
    fn build_client() -> Result<reqwest::Client, AppError> {
        reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
            .build()
            .map_err(|e| AppError::Database(format!("HTTP 客户端创建失败: {}", e)))
    }

    /// 新浪 API 返回 GBK 编码，需要手动解码为 UTF-8
    fn decode_sina_response(bytes: &[u8]) -> String {
        // 先尝试 UTF-8（部分接口可能返回 UTF-8）
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }
        // GBK 解码
        let (cow, _, _) = encoding_rs::GBK.decode(bytes);
        cow.into_owned()
    }

    // ── 实时行情 (新浪 hq.sinajs.cn) ──

    pub async fn get_quote(symbol: &str) -> Result<MarketQuote, AppError> {
        let client = Self::build_client()?;
        let url = format!("https://hq.sinajs.cn/list={}", symbol);
        let bytes = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Database(format!("读取行情响应失败: {}", e)))?;
        let text = Self::decode_sina_response(&bytes);

        Self::parse_sina_quote(symbol, &text)
    }

    pub async fn get_quotes(symbols: &[String]) -> Result<Vec<MarketQuote>, AppError> {
        let client = Self::build_client()?;
        let symbols_str = symbols.join(",");
        let url = format!("https://hq.sinajs.cn/list={}", symbols_str);
        let bytes = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Database(format!("读取行情响应失败: {}", e)))?;
        let text = Self::decode_sina_response(&bytes);

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

    // ── K 线数据 ──
    // 股票: 腾讯 fqkline API (稳定可靠)
    // 期货: 新浪期货 K 线 JSONP (nf_ 前缀格式)

    pub async fn get_kline(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let is_stock = symbol.starts_with("sh") || symbol.starts_with("sz");

        if is_stock {
            Self::fetch_stock_kline_tencent(symbol, period, count).await
        } else {
            // 期货: 尝试腾讯，失败则返回空
            Self::fetch_futures_kline(symbol, period, count).await
        }
    }

    /// 股票K线 — 腾讯财经 API
    /// 格式: [日期, 开盘, 收盘, 最高, 最低, 成交量]
    async fn fetch_stock_kline_tencent(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;
        let kline_type = match period {
            "1m" => "m1",
            "5m" => "m5",
            "15m" => "m15",
            "30m" => "m30",
            "60m" => "m60",
            "day" | "daily" => "day",
            _ => "day",
        };

        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},{},,,{},qfq",
            symbol, kline_type, count
        );

        let resp_text = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Database(format!("K线请求失败: {}", e)))?
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取K线失败: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| AppError::Database(format!("K线JSON解析失败: {}", e)))?;

        let data = json.get("data").and_then(|d| d.get(symbol));
        let Some(data) = data else {
            return Ok(Vec::new());
        };

        // qfqday 或 day
        let klines_arr = data
            .get("qfqday")
            .or_else(|| data.get("day"))
            .and_then(|v| v.as_array());

        let Some(arr) = klines_arr else {
            return Ok(Vec::new());
        };

        let mut klines = Vec::new();
        for item in arr {
            if let Some(row) = item.as_array() {
                if row.len() >= 6 {
                    klines.push(KlineData {
                        timestamp: row[0].as_str().unwrap_or("").to_string(),
                        open: row[1].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        close: row[2].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        high: row[3].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        low: row[4].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                        volume: row[5].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0),
                    });
                }
            }
        }

        Ok(klines)
    }

    /// 期货K线 — 新浪期货 JSONP API
    async fn fetch_futures_kline(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;

        let scale = match period {
            "1m" => "5",
            "5m" => "15",
            "15m" => "30",
            "30m" => "60",
            "60m" => "120",
            _ => "1440",
        };

        // 新浪期货K线: 尝试多种 symbol 格式
        // 1. 直接用传入的 (如 nf_RB0)
        // 2. 不带 nf_ 前缀的
        let symbols_to_try = if symbol.starts_with("nf_") {
            vec![symbol.to_string()]
        } else {
            vec![
                format!("nf_{}", symbol),
                symbol.to_string(),
            ]
        };

        for sym in &symbols_to_try {
            let url = format!(
                "https://finance.sina.com.cn/futures/api/jsonp.php/IO.XSRV2.CallbackList/xxx/CfuturesApiService.getKLineData?symbol={}&scale={}&datalen={}",
                sym, scale, count
            );

            if let Ok(resp) = client
                .get(&url)
                .header("Referer", "https://finance.sina.com.cn")
                .send()
                .await
            {
                if let Ok(bytes) = resp.bytes().await {
                    let text = Self::decode_sina_response(&bytes);
                    if let Ok(klines) = Self::parse_kline_jsonp(&text) {
                        if !klines.is_empty() {
                            return Ok(klines);
                        }
                    }
                }
            }
        }

        // 所有尝试都失败，返回空
        Ok(Vec::new())
    }

    // ── 搜索 ──

    pub async fn search_symbol_async(keyword: &str) -> Vec<SymbolInfo> {
        let mut results = Vec::new();

        if let Ok(client) = Self::build_client() {
            // type 留空 = 搜索全市场（A股+港股+期货+指数）
            let url = format!(
                "https://suggest3.sinajs.cn/suggest/type=&key={}&name=suggestdata",
                keyword
            );
            if let Ok(resp) = client
                .get(&url)
                .header("Referer", "https://finance.sina.com.cn")
                .send()
                .await
            {
                if let Ok(bytes) = resp.bytes().await {
                    let text = Self::decode_sina_response(&bytes);
                    results.extend(Self::parse_sina_suggest(&text));
                }
            }
        }

        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.symbol.clone()));
        results
    }

    pub fn search_symbol(keyword: &str) -> Vec<SymbolInfo> {
        Self::search_futures_local(keyword)
    }

    fn search_futures_local(keyword: &str) -> Vec<SymbolInfo> {
        // (搜索关键字, 品种名, 实时行情symbol, K线symbol)
        let futures_varieties: &[(&str, &str, &str)] = &[
            ("rb", "螺纹钢", "nf_RB0"), ("hc", "热卷", "nf_HC0"),
            ("i", "铁矿石", "nf_I0"), ("j", "焦炭", "nf_J0"),
            ("jm", "焦煤", "nf_JM0"), ("ZC", "动力煤", "nf_ZC0"),
            ("cu", "沪铜", "nf_CU0"), ("al", "沪铝", "nf_AL0"),
            ("zn", "沪锌", "nf_ZN0"), ("pb", "沪铅", "nf_PB0"),
            ("ni", "沪镍", "nf_NI0"), ("sn", "沪锡", "nf_SN0"),
            ("ss", "不锈钢", "nf_SS0"),
            ("au", "沪金", "nf_AU0"), ("ag", "沪银", "nf_AG0"),
            ("IF", "沪深300期货", "nf_IF0"), ("IC", "中证500期货", "nf_IC0"),
            ("IH", "上证50期货", "nf_IH0"), ("IM", "中证1000期货", "nf_IM0"),
            ("m", "豆粕", "nf_M0"), ("y", "豆油", "nf_Y0"),
            ("a", "豆一", "nf_A0"), ("b", "豆二", "nf_B0"),
            ("p", "棕榈油", "nf_P0"), ("OI", "菜油", "nf_OI0"),
            ("c", "玉米", "nf_C0"), ("cs", "淀粉", "nf_CS0"),
            ("CF", "棉花", "nf_CF0"), ("SR", "白糖", "nf_SR0"),
            ("AP", "苹果", "nf_AP0"), ("CJ", "红枣", "nf_CJ0"),
            ("TA", "PTA", "nf_TA0"), ("MA", "甲醇", "nf_MA0"),
            ("FG", "玻璃", "nf_FG0"), ("SA", "纯碱", "nf_SA0"),
            ("EG", "乙二醇", "nf_EG0"), ("PF", "短纤", "nf_PF0"),
            ("UR", "尿素", "nf_UR0"),
            ("SP", "纸浆", "nf_SP0"), ("FU", "燃油", "nf_FU0"),
            ("bu", "沥青", "nf_BU0"), ("pg", "液化气", "nf_PG0"),
            ("eb", "苯乙烯", "nf_EB0"), ("v", "PVC", "nf_V0"),
            ("pp", "聚丙烯", "nf_PP0"), ("l", "塑料", "nf_L0"),
            ("T", "十年国债", "nf_T0"), ("TF", "五年国债", "nf_TF0"),
            ("TS", "两年国债", "nf_TS0"), ("TL", "三十年国债", "nf_TL0"),
        ];

        let kw_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        for &(code, name, contract) in futures_varieties {
            if code.to_lowercase().contains(&kw_lower)
                || name.contains(keyword)
            {
                results.push(SymbolInfo {
                    symbol: contract.to_string(),
                    name: name.to_string(),
                    market_type: "futures".to_string(),
                });
            }
        }

        results
    }

    /// 新浪 suggest: type 编码
    /// 11=沪A, 12=深A, 31=港股, 33=港股指数, 41=美股,
    /// 81=场内基金, 87=期货主力品种, 103=期货合约, 109=期权合约
    fn parse_sina_suggest(text: &str) -> Vec<SymbolInfo> {
        let text = text.trim();
        let Some(eq_pos) = text.find('=') else {
            return Vec::new();
        };
        let value = text[eq_pos + 1..].trim_matches('"').trim_matches(';').trim();
        if value.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for item in value.split('|') {
            let fields: Vec<&str> = item.split(',').collect();
            if fields.len() < 5 {
                continue;
            }

            let type_code = fields[1].trim().parse::<u32>().unwrap_or(0);
            let full_code = fields[3].trim();
            let name = fields[4].trim();

            if full_code.is_empty() || name.is_empty() {
                continue;
            }

            let (symbol, market_type) = match type_code {
                // A股
                11 | 12 => {
                    let sym = if full_code.starts_with("sh") || full_code.starts_with("sz") {
                        full_code.to_string()
                    } else if full_code.starts_with("s_sh") {
                        format!("sh{}", &full_code[4..])
                    } else if full_code.starts_with("s_sz") {
                        format!("sz{}", &full_code[4..])
                    } else {
                        continue;
                    };
                    (sym, "stock".to_string())
                }
                // 港股
                31 | 33 => {
                    // 港股代码如 00700, hsi, hstech
                    (format!("hk{}", full_code), "hk_stock".to_string())
                }
                // 美股
                41 => (full_code.to_string(), "us_stock".to_string()),
                // 期货主力品种 (rb0, lc0 等)
                87 => {
                    // 加 nf_ 前缀用于实时行情
                    let code = full_code.trim_start_matches("nf_");
                    (format!("nf_{}", code.to_uppercase()), "futures".to_string())
                }
                // 期货具体合约 (rb2609, lc2609 等)
                103 => (full_code.to_string(), "futures".to_string()),
                // 跳过期权等
                _ => continue,
            };

            results.push(SymbolInfo {
                symbol,
                name: name.to_string(),
                market_type,
            });
        }

        results
    }

    // ── 订阅管理 ──

    pub fn start_subscription(symbols: Vec<String>, interval_ms: u64) {
        if let Ok(mut sub) = SUBSCRIPTION.lock() {
            *sub = Some(SubscriptionState { symbols, interval_ms });
        }
    }

    pub fn stop_subscription() {
        if let Ok(mut sub) = SUBSCRIPTION.lock() {
            *sub = None;
        }
    }

    // ── 解析器 ──

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

        let value_start = eq_pos + 2;
        let value_end = line.rfind('"')?;
        let value = &line[value_start..value_end];
        if value.is_empty() {
            return None;
        }

        let fields: Vec<&str> = value.split(',').collect();
        if fields.len() < 10 {
            return None;
        }

        let name = fields[0].to_string();
        let (open, prev_close, current, high, low, bid, ask, volume, amount) = (
            fields[1].parse().unwrap_or(0.0),
            fields[2].parse().unwrap_or(0.0),
            fields[3].parse().unwrap_or(0.0),
            fields[4].parse().unwrap_or(0.0),
            fields[5].parse().unwrap_or(0.0),
            fields[6].parse().unwrap_or(0.0),
            fields[7].parse().unwrap_or(0.0),
            fields[8].parse().unwrap_or(0.0),
            fields[9].parse().unwrap_or(0.0),
        );

        let change = if prev_close != 0.0 { current - prev_close } else { 0.0 };
        let change_pct = if prev_close != 0.0 {
            (current - prev_close) / prev_close * 100.0
        } else {
            0.0
        };

        // 股票: fields[30]=日期, fields[31]=时间
        // 商品期货(nf_前缀): fields[17]=日期
        // 股指期货(nf_前缀): fields[33]=日期, fields[34]=时间
        let timestamp = if fields.len() > 31 && !fields[30].is_empty() && fields[30].contains('-') {
            format!("{} {}", fields[30], fields[31])
        } else if fields.len() > 34 && !fields[33].is_empty() && fields[33].contains('-') {
            format!("{} {}", fields[33], fields[34])
        } else if fields.len() > 17 && !fields[17].is_empty() && fields[17].contains('-') {
            format!("{} 00:00:00", fields[17])
        } else {
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        };

        Some(MarketQuote {
            symbol, name, open, prev_close, current, high, low, bid, ask, volume, amount, change, change_pct, timestamp,
        })
    }

    fn parse_kline_jsonp(text: &str) -> Result<Vec<KlineData>, AppError> {
        let text = text.trim();
        let json_str = if let Some(start) = text.find('(') {
            if let Some(end) = text.rfind(')') {
                &text[start + 1..end]
            } else {
                text
            }
        } else {
            text
        };

        let items: Vec<HashMap<String, serde_json::Value>> =
            serde_json::from_str(json_str).unwrap_or_default();

        Ok(Self::extract_klines(&items))
    }

    fn extract_klines(items: &[HashMap<String, serde_json::Value>]) -> Vec<KlineData> {
        let mut klines = Vec::new();
        for item in items {
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
        klines
    }
}
