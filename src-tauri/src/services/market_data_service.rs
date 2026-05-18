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

    /// 新浪 API 返回 GBK，需手动解码
    fn decode_sina(bytes: &[u8]) -> String {
        if let Ok(s) = std::str::from_utf8(bytes) {
            return s.to_string();
        }
        let (cow, _, _) = encoding_rs::GBK.decode(bytes);
        cow.into_owned()
    }

    /// 将内部 symbol 转为新浪行情 API 格式
    fn to_sina_quote_symbol(symbol: &str) -> String {
        // A股: sh600519, sz000001 → 原样
        // 港股: hk00700 → rt_hk00700
        // 美股: usTME → gb_tme (小写)
        // 期货: nf_LC0 → 原样
        if let Some(code) = symbol.strip_prefix("hk") {
            format!("rt_hk{}", code)
        } else if let Some(code) = symbol.strip_prefix("us") {
            format!("gb_{}", code.to_lowercase())
        } else {
            symbol.to_string()
        }
    }

    // ── 实时行情 ──

    pub async fn get_quote(symbol: &str) -> Result<MarketQuote, AppError> {
        let sina_sym = Self::to_sina_quote_symbol(symbol);
        let client = Self::build_client()?;
        let url = format!("https://hq.sinajs.cn/list={}", sina_sym);
        let bytes = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Database(format!("读取行情失败: {}", e)))?;
        let text = Self::decode_sina(&bytes);
        Self::parse_sina_quote(symbol, &sina_sym, &text)
    }

    pub async fn get_quotes(symbols: &[String]) -> Result<Vec<MarketQuote>, AppError> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }
        let client = Self::build_client()?;

        // 按市场分组请求（新浪对不同市场用不同前缀）
        let mut all_quotes = Vec::new();

        // 批量：把所有内部 symbol 转成新浪格式，一次请求
        let sina_syms: Vec<String> = symbols.iter().map(|s| Self::to_sina_quote_symbol(s)).collect();
        let sina_str = sina_syms.join(",");
        let url = format!("https://hq.sinajs.cn/list={}", sina_str);
        let bytes = client
            .get(&url)
            .header("Referer", "https://finance.sina.com.cn")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("行情请求失败: {}", e)))?
            .bytes()
            .await
            .map_err(|e| AppError::Database(format!("读取行情失败: {}", e)))?;
        let text = Self::decode_sina(&bytes);

        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("var hq_str_") {
                if let Some(quote) = Self::parse_sina_line(line, symbols.get(i).map(|s| s.as_str()).unwrap_or("")) {
                    all_quotes.push(quote);
                }
            }
        }
        Ok(all_quotes)
    }

    // ── K 线数据 ──

    pub async fn get_kline(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        if symbol.starts_with("hk") {
            Self::fetch_kline_kline_api(symbol, period, count).await
        } else if symbol.starts_with("us") {
            Self::fetch_us_kline_yahoo(symbol, period, count).await
        } else if symbol.starts_with("sh") || symbol.starts_with("sz") {
            Self::fetch_stock_kline_tencent(symbol, period, count).await
        } else {
            Self::fetch_futures_kline_eastmoney(symbol, period, count).await
        }
    }

    /// A股 K线 — 腾讯 fqkline API (支持前复权)
    /// 键名: qfqday, qfqweek, qfqmonth
    async fn fetch_stock_kline_tencent(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;
        let kline_type = match period {
            "1m" => "m1", "5m" => "m5", "15m" => "m15",
            "30m" => "m30", "60m" => "m60",
            "day" | "daily" => "day", "week" => "week", "month" => "month",
            _ => "day",
        };

        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/fqkline/get?param={},{},,,{},qfq",
            symbol, kline_type, count
        );

        let resp = client.get(&url).send().await
            .map_err(|e| AppError::Database(format!("K线请求失败: {}", e)))?
            .text().await
            .map_err(|e| AppError::Database(format!("读取K线失败: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&resp)
            .map_err(|e| AppError::Database(format!("K线JSON解析失败: {}", e)))?;

        let data = json.get("data").and_then(|d| d.get(symbol));
        let Some(data) = data else { return Ok(Vec::new()) };

        // 按周期选择正确的键名
        let key = match period {
            "week" => "qfqweek",
            "month" => "qfqmonth",
            _ => "qfqday",
        };

        let arr = data.get(key).or_else(|| data.get("day")).and_then(|v| v.as_array());
        let Some(arr) = arr else { return Ok(Vec::new()) };

        Ok(Self::parse_tencent_kline_array(arr))
    }

    /// 港股/美股 K线 — 腾讯 kline API (不复权)
    /// 键名: day, week (无前缀)
    async fn fetch_kline_kline_api(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;
        let kline_type = match period {
            "1m" => "m1", "5m" => "m5", "15m" => "m15",
            "30m" => "m30", "60m" => "m60",
            "day" | "daily" => "day", "week" => "week", "month" => "month",
            _ => "day",
        };

        let url = format!(
            "https://web.ifzq.gtimg.cn/appstock/app/kline/kline?param={},{},,,{}",
            symbol, kline_type, count
        );

        let resp = client.get(&url).send().await
            .map_err(|e| AppError::Database(format!("K线请求失败: {}", e)))?
            .text().await
            .map_err(|e| AppError::Database(format!("读取K线失败: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&resp)
            .map_err(|e| AppError::Database(format!("K线JSON解析失败: {}", e)))?;

        let data = json.get("data").and_then(|d| d.get(symbol));
        let Some(data) = data else { return Ok(Vec::new()) };

        let key = match period {
            "week" => "week",
            "month" => "month",
            _ => "day",
        };

        let arr = data.get(key).and_then(|v| v.as_array());
        let Some(arr) = arr else { return Ok(Vec::new()) };

        Ok(Self::parse_tencent_kline_array(arr))
    }

    /// 美股 K线 — Yahoo Finance API
    /// symbol 格式: usTME → Yahoo: TME
    async fn fetch_us_kline_yahoo(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;

        // 去掉 "us" 前缀，转为 Yahoo symbol
        let yahoo_sym = symbol.strip_prefix("us").unwrap_or(symbol);

        let interval = match period {
            "5m" => "5m",
            "15m" => "15m",
            "30m" => "30m",
            "60m" => "1h",
            "day" | "daily" => "1d",
            "week" => "1wk",
            "month" => "1mo",
            _ => "1d",
        };

        // 根据周期选择合适的 range（确保获取足够的数据量）
        let range = match period {
            "5m" | "15m" | "30m" | "60m" => "1mo",
            "day" => "1y",
            "week" => "5y",
            "month" => "max",
            _ => "1y",
        };

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}",
            yahoo_sym, range, interval
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("Yahoo K线请求失败: {}", e)))?
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取Yahoo K线失败: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&resp)
            .map_err(|e| AppError::Database(format!("Yahoo K线JSON解析失败: {}", e)))?;

        let result = json
            .get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first());

        let Some(result) = result else { return Ok(Vec::new()) };

        let timestamps = result.get("timestamp").and_then(|t| t.as_array());
        let indicators = result.get("indicators").and_then(|i| i.get("quote"));
        let Some(timestamps) = timestamps else { return Ok(Vec::new()) };
        let Some(indicators) = indicators else { return Ok(Vec::new()) };
        let Some(quote) = indicators.as_array().and_then(|a| a.first()) else {
            return Ok(Vec::new());
        };

        let opens = quote.get("open").and_then(|v| v.as_array());
        let highs = quote.get("high").and_then(|v| v.as_array());
        let lows = quote.get("low").and_then(|v| v.as_array());
        let closes = quote.get("close").and_then(|v| v.as_array());
        let volumes = quote.get("volume").and_then(|v| v.as_array());

        let mut klines = Vec::new();
        for i in 0..timestamps.len() {
            let ts = timestamps.get(i).and_then(|v| v.as_i64()).unwrap_or(0);
            let timestamp = chrono::DateTime::from_timestamp(ts, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default();

            let get_f = |arr: Option<&Vec<serde_json::Value>>, idx: usize| -> f64 {
                arr.and_then(|a| a.get(idx))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };

            klines.push(KlineData {
                timestamp,
                open: get_f(opens.clone(), i),
                high: get_f(highs.clone(), i),
                low: get_f(lows.clone(), i),
                close: get_f(closes.clone(), i),
                volume: get_f(volumes.clone(), i),
            });
        }

        // 只取最后 count 条
        let total = klines.len();
        if total > count as usize {
            klines = klines.into_iter().skip(total - count as usize).collect();
        }

        Ok(klines)
    }

    fn parse_tencent_kline_array(arr: &[serde_json::Value]) -> Vec<KlineData> {
        let mut klines = Vec::new();
        for item in arr {
            let row = match item.as_array() {
                Some(r) => r,
                None => continue,
            };
            if row.len() < 6 {
                continue;
            }
            let get_f = |i: usize| -> f64 {
                row[i].as_str().and_then(|s| s.parse().ok())
                    .or_else(|| row[i].as_f64())
                    .unwrap_or(0.0)
            };
            klines.push(KlineData {
                timestamp: row[0].as_str().unwrap_or("").to_string(),
                open: get_f(1),
                close: get_f(2),
                high: get_f(3),
                low: get_f(4),
                volume: get_f(5),
            });
        }
        klines
    }

    /// 期货K线 — 东方财富
    /// symbol 格式: nf_LC0(主连) → 东财secid: 225.lcm
    /// symbol 格式: nf_M2609(具体合约) → 东财secid: 114.m2609
    async fn fetch_futures_kline_eastmoney(
        symbol: &str,
        period: &str,
        count: u32,
    ) -> Result<Vec<KlineData>, AppError> {
        let client = Self::build_client()?;

        let raw = symbol.strip_prefix("nf_").unwrap_or(symbol);
        let prefix = raw.trim_end_matches(|c: char| c.is_ascii_digit()).to_lowercase();
        let market = Self::futures_market_code(&prefix);

        // 判断是否是主力合约(末尾为0)还是具体合约(包含合约月份)
        let em_code = if raw.ends_with('0') && !raw.chars().take(raw.len() - 1).any(|c| c.is_ascii_digit()) {
            // 主力合约: LC0 → lcm
            format!("{}m", prefix)
        } else {
            // 具体合约: M2609 → m2609, RB2606 → rb2606
            raw.to_lowercase()
        };
        let secid = format!("{}.{}", market, em_code);

        let klt = match period {
            "5m" => "5",
            "15m" => "15",
            "30m" => "30",
            "60m" => "60",
            "day" | "daily" => "101",
            "week" => "102",
            "month" => "103",
            _ => "101",
        };

        let url = format!(
            "https://push2his.eastmoney.com/api/qt/stock/kline/get?secid={}&fields1=f1,f2,f3,f4,f5,f6&fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f62,f63,f64,f65&klt={}&fqt=0&lmt={}&end=20500101",
            secid, klt, count
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| AppError::Database(format!("东财期货K线请求失败: {}", e)))?
            .text()
            .await
            .map_err(|e| AppError::Database(format!("读取东财期货K线失败: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&resp)
            .map_err(|e| AppError::Database(format!("东财期货K线JSON解析失败: {}", e)))?;

        let Some(klines_arr) = json
            .get("data")
            .and_then(|d| d.get("klines"))
            .and_then(|v| v.as_array())
        else {
            return Ok(Vec::new());
        };

        let mut klines = Vec::new();
        for item in klines_arr {
            let line = item.as_str().unwrap_or("");
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 6 {
                continue;
            }
            let parse_f = |s: &str| -> f64 { s.parse().unwrap_or(0.0) };
            // 字段顺序: 日期, open, close, high, low, volume, ...
            klines.push(KlineData {
                timestamp: parts[0].to_string(),
                open: parse_f(parts[1]),
                close: parse_f(parts[2]),
                high: parse_f(parts[3]),
                low: parse_f(parts[4]),
                volume: parse_f(parts[5]),
            });
        }

        Ok(klines)
    }

    /// 品种前缀 → 东财交易所市场编号
    /// 上期所(SHFE)=113, 大商所(DCE)=114, 郑商所(CZCE)=115, 广期所(GFEX)=225, 中金所(CFFEX)=8
    fn futures_market_code(prefix: &str) -> u32 {
        match prefix {
            // 上期所
            "ag" | "au" | "cu" | "al" | "zn" | "pb" | "ni" | "sn" |
            "rb" | "hc" | "ss" | "bu" | "ru" | "sp" | "fu" | "wr" |
            "nr" | "bc" | "ad" => 113,
            // 大商所
            "a" | "b" | "c" | "cs" | "m" | "y" | "p" | "jd" |
            "l" | "v" | "pp" | "j" | "jm" | "i" | "eg" | "eb" |
            "pg" | "lh" | "fb" | "bb" | "rr" | "bz" | "lg" => 114,
            // 郑商所
            "wh" | "pm" | "ri" | "rs" | "sr" | "cf" | "ta" | "oi" |
            "ma" | "fg" | "sf" | "sm" | "zc" | "sa" | "ur" | "sh" |
            "ap" | "cj" | "pk" | "cy" | "rm" => 115,
            // 中金所
            "if" | "ic" | "ih" | "im" | "tf" | "ts" | "tl" | "t" => 8,
            // 广期所
            "lc" | "si" => 225,
            // 默认（尝试上期所）
            _ => 113,
        }
    }

    // ── 搜索 ──

    pub async fn search_symbol_async(keyword: &str) -> Vec<SymbolInfo> {
        let mut results = Vec::new();

        // 新浪搜索 — A股、港股、美股
        if let Ok(client) = Self::build_client() {
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
                    let text = Self::decode_sina(&bytes);
                    results = Self::parse_sina_suggest(&text);
                }
            }
        }

        // 东财搜索 — 补充期货品种
        if let Ok(client) = Self::build_client() {
            let encoded = urlencoding::encode(keyword);
            let url = format!(
                "https://searchapi.eastmoney.com/api/suggest/get?input={}&type=14&token=D43BF722C8E6B4AA2A199F24E7E0934E&count=20",
                encoded
            );
            if let Ok(resp) = client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                .send()
                .await
            {
                if let Ok(text) = resp.text().await {
                    let futures = Self::parse_eastmoney_futures(&text);
                    results.extend(futures);
                }
            }
        }

        // 去重
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.symbol.clone()));
        results
    }

    /// 解析东财搜索结果，只提取国内期货品种(SecurityType=12)
    fn parse_eastmoney_futures(text: &str) -> Vec<SymbolInfo> {
        let json: serde_json::Value = serde_json::from_str(text).unwrap_or(serde_json::Value::Null);
        let data = json.get("QuotationCodeTable").and_then(|d| d.get("Data"));
        let Some(data) = data else { return Vec::new() };
        let Some(arr) = data.as_array() else { return Vec::new() };

        let mut results = Vec::new();
        for item in arr {
            let stype = item.get("SecurityType").and_then(|v| v.as_str()).unwrap_or("");
            if stype != "12" { continue }  // 只取国内期货

            let quote_id = item.get("QuoteID").and_then(|v| v.as_str()).unwrap_or("");
            let name = item.get("Name").and_then(|v| v.as_str()).unwrap_or("");

            // QuoteID 格式: "114.m2609" → 品种代码 = "m2609"
            let dot_pos = quote_id.find('.');
            let Some(dot_pos) = dot_pos else { continue };
            let em_code = &quote_id[dot_pos + 1..];
            if em_code.is_empty() { continue }

            // 内部 symbol: nf_M2609 (品种代码大写 + nf_ 前缀)
            // 但主连/次主连代码(如rbm, rbs)需要特殊处理: rbm → nf_RB0 (主连)
            let symbol = if em_code.ends_with('m') || em_code.ends_with('s') {
                // 主连(rbm)或次主连(rbs) → 转为主力合约格式 nf_RB0
                let prefix = em_code.trim_end_matches('m').trim_end_matches('s');
                format!("nf_{}0", prefix.to_uppercase())
            } else {
                format!("nf_{}", em_code.to_uppercase())
            };

            // 如果名称包含"主连"/"次主连"，简化名称
            let display_name = if name.contains("主连") {
                name.replace("主连", "主力")
            } else if name.contains("次主连") {
                name.replace("次主连", "次主力")
            } else {
                name.to_string()
            };

            results.push(SymbolInfo {
                symbol,
                name: display_name,
                market_type: "futures".to_string(),
            });
        }
        results
    }

    pub fn search_symbol(keyword: &str) -> Vec<SymbolInfo> {
        // 同步版，仅本地
        Vec::new()
    }

    /// type 编码:
    /// 11=沪A, 12=深A, 31=港股, 33=港股指数, 41=美股,
    /// 87=期货品种(主力), 103=期货合约
    fn parse_sina_suggest(text: &str) -> Vec<SymbolInfo> {
        let text = text.trim();
        let Some(eq_pos) = text.find('=') else { return Vec::new() };
        let value = text[eq_pos + 1..].trim_matches('"').trim_matches(';').trim();
        if value.is_empty() { return Vec::new() }

        let mut results = Vec::new();
        'outer: for item in value.split('|') {
            let fields: Vec<&str> = item.split(',').collect();
            if fields.len() < 5 { continue }
            let type_code = fields[1].trim().parse::<u32>().unwrap_or(0);
            let full_code = fields[3].trim();
            let name = fields[4].trim();
            if full_code.is_empty() || name.is_empty() { continue }

            let (symbol, market_type) = match type_code {
                11 | 12 => {
                    let sym = if full_code.starts_with("sh") || full_code.starts_with("sz") {
                        full_code.to_string()
                    } else if full_code.starts_with("s_sh") {
                        format!("sh{}", &full_code[4..])
                    } else if full_code.starts_with("s_sz") {
                        format!("sz{}", &full_code[4..])
                    } else { continue 'outer };
                    (sym, "stock".to_string())
                }
                31 | 33 => (format!("hk{}", full_code), "hk_stock".to_string()),
                41 => (format!("us{}", full_code.to_uppercase()), "us_stock".to_string()),
                87 => {
                    // 期货主力: lc0 → nf_LC0 (大写品种代码)
                    let code = full_code.trim_start_matches("nf_");
                    (format!("nf_{}", code.to_uppercase()), "futures".to_string())
                }
                103 => (full_code.to_string(), "futures".to_string()),
                _ => continue,
            };

            results.push(SymbolInfo { symbol, name: name.to_string(), market_type });
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
        if let Ok(mut sub) = SUBSCRIPTION.lock() { *sub = None }
    }

    // ── 解析器 ──

    fn parse_sina_quote(internal_sym: &str, sina_sym: &str, text: &str) -> Result<MarketQuote, AppError> {
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with(&format!("var hq_str_{}=", sina_sym)) {
                return Self::parse_sina_line(line, internal_sym).ok_or_else(|| {
                    AppError::Database(format!("解析行情失败: {}", internal_sym))
                });
            }
        }
        Err(AppError::NotFound(format!("未找到行情: {}", internal_sym)))
    }

    /// 解析新浪行情行，统一输出
    /// A股: 32+ 字段，fields[0]=名称, [1]=开, [2]=昨收, [3]=当前, [4]=高, [5]=低, [6]=买, [7]=卖, [8]=量, [9]=额, [30]=日, [31]=时间
    /// 港股: 12+ 字段, fields[0]=英文名, [1]=中文名, [2]=开, [3]=高, [4]=低, [5]=昨收, [6]=当前价, [10]=涨跌, [11]=涨跌幅, [12]=成交量
    /// 美股: 12+ 字段, fields[0]=中文名, [1]=当前价, [2]=涨跌额, [3]=日期时间, [4]=涨跌额, [5]=高, [6]=52周高, [7]=低, [8]=成交量, [9]=昨收
    /// 期货(nf_): 类似A股但字段可能不同
    fn parse_sina_line(line: &str, internal_sym: &str) -> Option<MarketQuote> {
        let eq_pos = line.find('=')?;
        let _raw_sym = line["var hq_str_".len()..eq_pos].to_string();
        let value_start = eq_pos + 2;
        let value_end = line.rfind('"')?;
        let value = &line[value_start..value_end];
        if value.is_empty() { return None }

        let fields: Vec<&str> = value.split(',').collect();

        // 检测是否为港股 (rt_hk 格式: fields[0]=英文名, fields[1]=中文名)
        let is_hk = internal_sym.starts_with("hk");
        let is_us = internal_sym.starts_with("us");

        let (name, open, prev_close, current, high, low, volume, amount, timestamp) = if is_hk && fields.len() >= 12 {
            let name = fields[1].to_string();
            let open = fields[2].parse().unwrap_or(0.0);
            let high = fields[3].parse().unwrap_or(0.0);
            let low = fields[4].parse().unwrap_or(0.0);
            let prev_close = fields[5].parse().unwrap_or(0.0);
            let current = fields[6].parse().unwrap_or(0.0);
            let volume = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let ts = fields.get(17).map(|s| s.to_string()).unwrap_or_default();
            (name, open, prev_close, current, high, low, volume, 0.0, ts)
        } else if is_us && fields.len() >= 10 {
            // 美股: fields[0]=名, [1]=价, [2]=涨跌, [3]=时间
            let name = fields[0].to_string();
            let current = fields[1].parse().unwrap_or(0.0);
            let change = fields[2].parse().unwrap_or(0.0);
            let prev_close = current - change;
            let high = fields.get(5).and_then(|s| s.parse().ok()).unwrap_or(current);
            let low = fields.get(7).and_then(|s| s.parse().ok()).unwrap_or(current);
            let open = fields.get(9).and_then(|s| s.parse().ok()).unwrap_or(current);
            let volume = fields.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let ts = fields.get(3).map(|s| s.to_string()).unwrap_or_default();
            (name, open, prev_close, current, high, low, volume, 0.0, ts)
        } else if fields.len() >= 10 {
            // A股/期货
            let name = fields[0].to_string();
            let open = fields[1].parse().unwrap_or(0.0);
            let prev_close = fields[2].parse().unwrap_or(0.0);
            let current = fields[3].parse().unwrap_or(0.0);
            let high = fields[4].parse().unwrap_or(0.0);
            let low = fields[5].parse().unwrap_or(0.0);
            let bid = fields.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let ask = fields.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let volume = fields.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let amount = fields.get(9).and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let ts = if fields.len() > 31 && fields[30].contains('-') {
                format!("{} {}", fields[30], fields[31])
            } else if fields.len() > 34 && fields[33].contains('-') {
                format!("{} {}", fields[33], fields[34])
            } else if fields.len() > 17 && fields[17].contains('-') {
                format!("{} 00:00:00", fields[17])
            } else {
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            };
            return Some(MarketQuote {
                symbol: internal_sym.to_string(), name, open, prev_close, current, high, low,
                bid, ask, volume, amount,
                change: if prev_close != 0.0 { current - prev_close } else { 0.0 },
                change_pct: if prev_close != 0.0 { (current - prev_close) / prev_close * 100.0 } else { 0.0 },
                timestamp: ts,
            });
        } else {
            return None;
        };

        let change = if prev_close != 0.0 { current - prev_close } else { 0.0 };
        let change_pct = if prev_close != 0.0 { (current - prev_close) / prev_close * 100.0 } else { 0.0 };

        Some(MarketQuote {
            symbol: internal_sym.to_string(), name, open, prev_close, current, high, low,
            bid: 0.0, ask: 0.0, volume, amount,
            change, change_pct, timestamp,
        })
    }

    fn parse_kline_jsonp(text: &str) -> Result<Vec<KlineData>, AppError> {
        let text = text.trim();
        let json_str = if let Some(start) = text.find('(') {
            if let Some(end) = text.rfind(')') { &text[start + 1..end] } else { text }
        } else { text };

        let items: Vec<HashMap<String, serde_json::Value>> =
            serde_json::from_str(json_str).unwrap_or_default();

        let mut klines = Vec::new();
        for item in &items {
            let get_f = |key: &str| -> f64 {
                item.get(key)
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
                    .or_else(|| item.get(key).and_then(|v| v.as_f64()))
                    .unwrap_or(0.0)
            };
            let ts = item.get("day").or_else(|| item.get("d"))
                .and_then(|v| v.as_str()).unwrap_or("").to_string();
            klines.push(KlineData {
                timestamp: ts,
                open: get_f("open"), high: get_f("high"),
                low: get_f("low"), close: get_f("close"), volume: get_f("volume"),
            });
        }
        Ok(klines)
    }
}
