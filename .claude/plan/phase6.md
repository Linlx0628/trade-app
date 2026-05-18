# Phase 6 开发规划：交易终端暗色主题 + 行情数据 + 多账户 + 自动备份

> 项目：策盈 TradeMind（Tauri V2 + Vue 3 + Rust）
> 版本：v0.6.0
> 编写日期：2026-05-15
> 基于：trade-app-rewrite.md Phase 6 + 用户反馈迭代

---

## 一、Phase 6 总体目标

Phase 6 聚焦四大方向：

1. **交易终端暗色主题** — 从当前的浅色/默认主题切换为专业交易终端暗色配色方案
2. **实时行情数据** — 接入免费行情 API，实现实时价格监控和 K 线图展示
3. **多账户管理** — 完善多账户 UI 和多用户登录支持
4. **自动备份方案** — 简单可靠的数据自动备份机制，防止用户忘记备份导致数据丢失

### 1.1 范围调整说明

| 原计划任务 | 决定 | 原因 |
|------------|------|------|
| 6.1 行情数据源 | ✅ 实现 | 核心功能 |
| 6.2 WebSocket 行情 | ✅ 实现 | 实时数据需要 |
| 6.3 K 线图组件 | ✅ 实现 | 技术分析需要 |
| 6.4 缠论信号检测 | ✅ 实现 | 核心策略功能 |
| 6.5 多账户 UI | ✅ 实现 | 多账户管理 |
| 6.6 多用户支持 | ✅ 实现 | 多用户登录 |
| 6.7 云同步服务 | ❌ 暂不实现 | 无云服务器可用 |
| 6.8 移动端适配 | ❌ 暂不实现 | 优先级后移 |
| **新增** 自动备份 | ✅ 实现 | 防止数据丢失 |
| **新增** 暗色主题 | ✅ 实现 | 专业交易终端视觉升级 |

---

## 二、技术架构总览

### 2.1 新增技术栈

| 层级 | 技术 | 说明 |
|------|------|------|
| 行情数据 | Sina Finance API (HTTP + WebSocket) | 免费、稳定、国内期货/股票实时行情 |
| K 线图 | ECharts (vue-echarts) | 已在规划中，K 线图组件 |
| 自动备份 | Rust 本地文件操作 | 定时备份 SQLite 数据库文件 |
| 暗色主题 | Tailwind CSS 4 CSS 变量 | 全局主题色系统 |

### 2.2 行情数据源选型

**选择：Sina Finance API（新浪财经）**

| 对比项 | Sina Finance | Tushare | 东方财富 |
|--------|-------------|---------|----------|
| 费用 | 免费 | 免费额度有限 | 免费（非官方） |
| 注册要求 | 无 | 需注册 | 无 |
| 实时行情 | ✅ HTTP + WebSocket | ❌ 仅历史 | ✅ 但不稳定 |
| 期货支持 | ✅ 完整 | ✅ 有限 | ✅ 完整 |
| 股票支持 | ✅ 完整 | ✅ 完整 | ✅ 完整 |
| 稳定性 | 高（老牌接口） | 高（官方） | 中 |
| 请求限制 | 宽松 | 有限流 | 宽松 |

**Sina Finance API 端点：**
- HTTP 行情：`https://hq.sinajs.cn/list=rb2510,m2501,sh600519`（期货、股票均支持）
- 历史数据：`https://finance.sina.com.cn/futures/api/jsonp.php/var/IO.XSRV2.CallbackList/xxx/CfuturesApiService.getCffexKLineData?symbol=rb2510&type=5`（K 线数据）
- WebSocket：`wss://hq.sinajs.cn` 或通过 HTTP 轮询实现（更稳定）

> **注意**：由于 Sina Finance 是非官方 API，需要做好降级和错误处理。HTTP 轮询方案（每 3-5 秒）作为主要实现方式，比 WebSocket 更稳定可靠。

### 2.3 自动备份方案选型

**选择：本地定时自动备份 + 云同步目录支持**

| 对比项 | 自建同步服务 | BaaS (Supabase) | **本地自动备份 + 云目录** |
|--------|-------------|-----------------|--------------------------|
| 需要服务器 | ✅ 需要 | 需要 | ❌ 不需要 |
| 实现复杂度 | 高 | 中 | **低** |
| 用户配置 | 复杂 | 中等 | **简单（选目录即可）** |
| 数据安全 | 依赖网络 | 依赖第三方 | **本地可控** |
| 跨设备同步 | 实时 | 实时 | **通过 iCloud/Dropbox 间接实现** |

**工作原理：**
1. 应用启动时自动备份当前数据库到用户配置的备份目录
2. 定时备份（默认每 2 小时），可配置间隔
3. 备份文件按日期命名：`trademind_backup_20260515_143000.db`
4. 自动清理旧备份（默认保留最近 30 份）
5. 用户可将备份目录设为 iCloud Drive / Dropbox / OneDrive 同步目录，实现间接跨设备同步

---

## 三、配色方案：交易终端暗色主题

### 3.1 核心色值定义

基于用户提供的参考图片，提取的专业交易终端配色方案：

```css
:root {
  /* === 背景层级 === */
  --bg-primary:      #0f151d;   /* 主背景 - 深海军蓝 */
  --bg-secondary:    #141c26;   /* 次要背景 - 卡片/面板 */
  --bg-tertiary:     #1a2433;   /* 三级背景 - 悬浮/选中 */
  --bg-overlay:      #0d121a;   /* 遮罩层背景 */

  /* === 文字层级 === */
  --text-primary:    #e8eaed;   /* 主文字 - 亮灰白 */
  --text-secondary:  #9aa5b4;   /* 次要文字 - 中灰 */
  --text-muted:      #5c6a7a;   /* 弱化文字 - 暗灰 */
  --text-inverse:    #0f151d;   /* 反色文字 */

  /* === 功能色 === */
  --color-profit:    #22bc53;   /* 盈利 - 翠绿 */
  --color-loss:      #e03f3c;   /* 亏损 - 正红 */
  --color-accent:    #cea43e;   /* 强调 - 琥珀金 */
  --color-accent-light: #ebc27e; /* 浅强调 - 淡金 */
  --color-warning:   #e8a735;   /* 警告 - 橙黄 */
  --color-info:      #3b82f6;   /* 信息 - 蓝 */

  /* === 边框与分割 === */
  --border-primary:  #1e2a3a;   /* 主边框 */
  --border-secondary:#2a3a4e;   /* 次要边框/悬浮边框 */
  --divider:         #1a2433;   /* 分割线 */

  /* === 交互状态 === */
  --hover-bg:        #1a2433;   /* 悬浮背景 */
  --active-bg:       #243044;   /* 按下背景 */
  --focus-ring:      #cea43e;   /* 焦点环 */

  /* === 图表色板 === */
  --chart-line:      #3b82f6;   /* 主线 */
  --chart-ma5:       #e8a735;   /* MA5 */
  --chart-ma10:      #3b82f6;   /* MA10 */
  --chart-ma20:      #9333ea;   /* MA20 */
  --chart-volume-up: #22bc53;   /* 成交量涨 */
  --chart-volume-down: #e03f3c; /* 成交量跌 */
}
```

### 3.2 Tailwind CSS 4 主题配置

修改 `src/assets/css/main.css` 中的 Tailwind 主题变量，将所有组件颜色映射到上述暗色方案。

### 3.3 需要修改的文件范围

| 文件 | 修改内容 |
|------|----------|
| `src/assets/css/main.css` | Tailwind 主题变量全面替换为暗色方案 |
| `src/components/ui/**/*.vue` | 所有 UI 组件适配暗色主题 |
| `src/components/layout/AppLayout.vue` | 侧边栏/顶栏暗色适配 |
| `src/views/**/*.vue` | 所有页面视图适配 |
| `src-tauri/tauri.conf.json` | 窗口背景色设置为 `#0f151d` |

---

## 四、模块一：自动备份系统

### 4.1 功能设计

**目标：** 零配置可用、用户可选增强的数据保护机制。

**核心功能：**
1. **启动备份** — 每次应用启动时自动创建备份
2. **定时备份** — 可配置间隔自动备份（默认 2 小时）
3. **关闭备份** — 应用正常关闭时创建备份
4. **备份目录** — 默认在应用数据目录下的 `backups/` 文件夹，用户可自定义
5. **备份轮转** — 自动清理超过 N 份的旧备份（默认 30 份）
6. **云同步友好** — 用户可将备份目录设为 iCloud/Dropbox 同步目录
7. **恢复引导** — 检测到异常退出时，下次启动提示恢复最近备份

### 4.2 后端实现

#### 4.2.1 文件结构

```
src-tauri/src/
  services/
    auto_backup_service.rs  -- 新增：自动备份服务
  commands/
    backup.rs               -- 新增：备份相关 Tauri Commands
  models/
    backup.rs               -- 新增：备份数据模型
```

#### 4.2.2 Rust 模型定义

```rust
pub struct BackupConfig {
    pub enabled: bool,               // 是否启用自动备份
    pub backup_dir: String,          // 备份目录路径（空则使用默认）
    pub interval_minutes: u64,       // 备份间隔（分钟），默认 120
    pub max_backups: u32,            // 最大备份数，默认 30
    pub backup_on_start: bool,       // 启动时备份，默认 true
    pub backup_on_close: bool,       // 关闭时备份，默认 true
}

pub struct BackupInfo {
    pub filename: String,            // 文件名
    pub file_size: u64,              // 文件大小（字节）
    pub created_at: String,          // 创建时间
    pub is_auto: bool,               // 是否自动备份
}

pub struct BackupStatus {
    pub last_backup_at: Option<String>,
    pub next_backup_at: Option<String>,
    pub total_backups: u32,
    pub backup_dir: String,
    pub total_size_mb: f64,
}
```

#### 4.2.3 Tauri Commands

```rust
#[tauri::command]
pub fn get_backup_config(state) -> Result<BackupConfig>

#[tauri::command]
pub fn update_backup_config(state, config: BackupConfig) -> Result<BackupConfig>

#[tauri::command]
pub fn perform_backup(state) -> Result<BackupInfo>
// 立即执行一次备份

#[tauri::command]
pub fn list_backups(state) -> Result<Vec<BackupInfo>>

#[tauri::command]
pub fn restore_from_backup(state, filename: String) -> Result<()>
// 从指定备份恢复

#[tauri::command]
pub fn delete_backup(state, filename: String) -> Result<()>

#[tauri::command]
pub fn get_backup_status(state) -> Result<BackupStatus>

#[tauri::command]
pub fn choose_backup_directory() -> Result<String>
// 打开目录选择对话框，返回用户选择的路径
```

#### 4.2.4 核心备份逻辑

```rust
impl AutoBackupService {
    /// 执行备份：复制 SQLite 文件到备份目录
    pub fn perform_backup(db_path: &Path, config: &BackupConfig) -> Result<BackupInfo> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("trademind_backup_{}.db", timestamp);
        let backup_path = PathBuf::from(&config.backup_dir).join(&filename);

        // 1. 确保备份目录存在
        fs::create_dir_all(&config.backup_dir)?;

        // 2. 使用 SQLite VACUUM INTO 命令创建一致性备份
        // （比直接复制文件更安全，不需要停止写入）
        // 或者使用 rusqlite 的 backup API

        // 3. 清理旧备份
        if config.max_backups > 0 {
            cleanup_old_backups(&config.backup_dir, config.max_backups)?;
        }

        Ok(backup_info)
    }

    /// 启动定时备份线程
    pub fn start_periodic_backup(app_handle: AppHandle, interval: Duration) {
        // 使用 tokio::spawn 启动定时任务
        // 每隔 interval 时间执行一次备份
    }

    /// 清理旧备份，保留最新的 N 份
    fn cleanup_old_backups(backup_dir: &Path, max_count: u32) -> Result<()> {
        // 列出目录下所有备份文件，按时间排序
        // 删除超过 max_count 的旧文件
    }

    /// 检测是否为异常退出后的首次启动
    pub fn detect_abnormal_shutdown(backup_dir: &Path) -> Option<BackupInfo> {
        // 检查是否存在 .backup_lock 文件
        // 如果存在，说明上次未正常关闭
    }
}
```

### 4.3 前端实现

#### 4.3.1 文件结构

```
src/
  views/settings/
    SettingsView.vue                  -- 修改：新增「数据保护」区域
    components/
      BackupConfigPanel.vue          -- 新增：自动备份配置面板
      BackupListPanel.vue            -- 新增：备份列表管理
      BackupStatusBanner.vue         -- 新增：备份状态横幅（可选）
```

#### 4.3.2 UI 交互流程

**备份配置面板：**
- 开关：启用/禁用自动备份
- 备份目录选择：显示当前路径 + 「更改目录」按钮
- 备份间隔：下拉选择（30分钟 / 1小时 / 2小时 / 4小时 / 8小时 / 24小时）
- 最大备份数：滑块或输入框（10-100）
- 启动备份开关 / 关闭备份开关
- 「立即备份」按钮
- 当前备份状态显示（上次备份时间、下次备份时间、总备份大小）

**备份列表：**
- 列出所有备份文件（时间、大小、类型）
- 每条记录有「恢复」和「删除」操作
- 恢复时弹出确认对话框（提示会覆盖当前数据）

**异常恢复提示：**
- 应用启动时检测到异常退出，弹出提示
- 显示最近备份信息
- 用户可选择「从备份恢复」或「忽略」

---

## 五、模块二：实时行情数据

### 5.1 数据源设计

**主要方案：Sina Finance HTTP 轮询**

采用 HTTP 轮询而非 WebSocket 的理由：
- Sina Finance 的 WebSocket 接口不稳定，经常断连
- HTTP 轮询实现简单、可靠、易于错误处理
- 3-5 秒轮询间隔对交易决策场景足够（非高频交易）
- 减少依赖（无需 `tokio-tungstenite`）

**API 格式：**
```
# 期货：品种代码格式 rb2510, m2501, IF2510 等
# 股票：sh600519, sz000001 等

GET https://hq.sinajs.cn/list=rb2510
Referer: https://finance.sina.com.cn
```

**返回格式（逗号分隔）：**
```
品种名称,开盘价,昨收,当前价,最高价,最低价,买一价,卖一价,成交量,成交额,...
```

### 5.2 后端实现

#### 5.2.1 文件结构

```
src-tauri/src/
  services/
    market_data_service.rs  -- 新增：行情数据服务
  commands/
    market_data.rs          -- 新增：行情数据 Tauri Commands
  models/
    market_data.rs          -- 新增：行情数据模型
```

#### 5.2.2 Rust 模型定义

```rust
pub struct MarketQuote {
    pub symbol: String,           // 品种代码
    pub name: String,             // 品种名称
    pub open: f64,                // 开盘价
    pub prev_close: f64,          // 昨收
    pub current: f64,             // 当前价
    pub high: f64,                // 最高
    pub low: f64,                 // 最低
    pub bid: f64,                 // 买一
    pub ask: f64,                 // 卖一
    pub volume: f64,              // 成交量
    pub amount: f64,              // 成交额
    pub change: f64,              // 涨跌额
    pub change_pct: f64,          // 涨跌幅%
    pub timestamp: String,        // 行情时间
}

pub struct KlineData {
    pub symbol: String,
    pub period: String,           // 1m/5m/15m/30m/60m/day/week/month
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub struct KlineRequest {
    pub symbol: String,
    pub period: String,           // K 线周期
    pub count: Option<u32>,       // 返回条数，默认 100
}

pub struct MarketSubscription {
    pub symbols: Vec<String>,     // 订阅的品种列表
    pub interval_ms: u64,         // 轮询间隔（毫秒），默认 3000
}
```

#### 5.2.3 Tauri Commands

```rust
#[tauri::command]
pub async fn get_quote(state, symbol: String) -> Result<MarketQuote>
// 获取单个品种实时行情

#[tauri::command]
pub async fn get_quotes(state, symbols: Vec<String>) -> Result<Vec<MarketQuote>>
// 批量获取多个品种行情

#[tauri::command]
pub async fn get_kline_data(state, request: KlineRequest) -> Result<Vec<KlineData>>
// 获取 K 线历史数据

#[tauri::command]
pub async fn subscribe_market(state, subscription: MarketSubscription) -> Result<()>
// 开始订阅行情（启动后台轮询线程）
// 通过 Tauri Event 定期推送行情数据到前端
// 事件名: "market-quote-update"

#[tauri::command]
pub async fn unsubscribe_market(state) -> Result<()>
// 停止行情订阅

#[tauri::command]
pub async fn search_symbol(state, keyword: String) -> Result<Vec<SymbolInfo>>
// 搜索品种代码/名称
```

#### 5.2.4 核心行情逻辑

```rust
impl MarketDataService {
    /// 解析 Sina Finance 行情数据
    fn parse_sina_quote(raw: &str) -> Result<MarketQuote> {
        // 解析 hq.sinajs.cn 返回的格式
        // 格式: var hq_str_rb2510="品种名,开盘,..."
    }

    /// 获取 K 线数据（新浪历史数据 API）
    async fn fetch_kline(symbol: &str, period: &str, count: u32) -> Result<Vec<KlineData>> {
        // 调用新浪 K 线 API
    }

    /// 启动行情订阅轮询
    pub fn start_subscription(app: &AppHandle, symbols: Vec<String>, interval_ms: u64) {
        // tokio::spawn 一个定时轮询任务
        // 每次轮询调用 get_quotes 获取最新数据
        // 通过 app.emit("market-quote-update", quotes) 推送到前端
    }
}
```

### 5.3 前端实现

#### 5.3.1 文件结构

```
src/
  stores/
    market.ts                     -- 新增：行情数据状态管理
  views/
    signal-monitor/               -- 新增：信号监控模块
      SignalMonitorView.vue       -- 新增：信号监控主页
      components/
        PriceTicker.vue           -- 新增：实时价格显示条
        KlineChart.vue            -- 新增：K 线图组件
        QuoteCard.vue             -- 新增：行情卡片
        SymbolSearch.vue          -- 新增：品种搜索
  components/
    shared/
      MarketTypeSwitch.vue        -- 修改：集成行情数据
  composables/
    useMarketData.ts              -- 新增：行情数据组合式函数
```

#### 5.3.2 Pinia Store（`stores/market.ts`）

```typescript
interface MarketState {
  subscriptions: string[]          // 订阅的品种列表
  quotes: Record<string, MarketQuote>  // 最新行情数据
  loading: boolean
  connected: boolean               // 是否正在接收行情
  klineData: KlineData[]           // 当前 K 线数据
  currentSymbol: string | null     // 当前查看的品种
}

// Actions
subscribe(symbols: string[])       // 订阅行情
unsubscribe()                      // 取消订阅
fetchKline(symbol, period, count)  // 获取 K 线数据
searchSymbol(keyword)              // 搜索品种
```

#### 5.3.3 K 线图组件设计

使用 ECharts 的 K 线图类型：

```typescript
// K 线图配置
const chartOption = {
  backgroundColor: '#0f151d',
  grid: { left: 60, right: 20, top: 30, bottom: 60 },
  xAxis: { type: 'category', data: timestamps, axisLine: { lineStyle: { color: '#1e2a3a' } } },
  yAxis: { type: 'value', axisLine: { lineStyle: { color: '#1e2a3a' } } },
  series: [
    {
      type: 'candlestick',
      data: klineData,  // [open, close, low, high]
      itemStyle: {
        color: '#22bc53',        // 阳线填充色
        color0: '#e03f3c',       // 阴线填充色
        borderColor: '#22bc53',  // 阳线边框
        borderColor0: '#e03f3c', // 阴线边框
      }
    },
    {
      type: 'line',
      name: 'MA5',
      lineStyle: { color: '#e8a735' }
    },
    // ... MA10, MA20
  ]
}
```

---

## 六、模块三：缠论信号检测

### 6.1 功能设计

基于实时 K 线数据，自动检测缠论关键信号：
- **笔（Bi）** — 相邻顶分型和底分型之间的连线
- **线段（Segment）** — 由至少 3 笔构成的趋势段
- **中枢（Pivot）** — 至少 3 个次级别线段重叠区间
- **买卖点** — 一买/一卖/二买/二卖/三买/三卖

### 6.2 后端实现

#### 6.2.1 文件结构

```
src-tauri/src/
  services/
    chanlun_service.rs      -- 新增：缠论分析服务
  models/
    signal_alert.rs         -- 新增：信号模型
```

#### 6.2.2 Rust 模型定义

```rust
pub struct Fractal {
    pub index: u32,              // K 线索引
    pub fractal_type: String,    // "top" | "bottom"
    pub value: f64,              // 极值
}

pub struct Bi {
    pub start_fractal: Fractal,
    pub end_fractal: Fractal,
    pub direction: String,       // "up" | "down"
}

pub struct Pivot {
    pub start_index: u32,
    pub end_index: u32,
    pub zg: f64,                 // 中枢上沿（ZG）
    pub zd: f64,                 // 中枢下沿（ZD）
    pub zz: f64,                 // 中枢中心值
}

pub struct ChanlunSignal {
    pub signal_type: String,     // "buy1" | "sell1" | "buy2" | "sell2" | "buy3" | "sell3"
    pub symbol: String,
    pub price: f64,
    pub pivot: Option<Pivot>,
    pub description: String,
    pub detected_at: String,
}

pub struct ChanlunAnalysis {
    pub symbol: String,
    pub fractals: Vec<Fractal>,
    pub bis: Vec<Bi>,
    pub pivots: Vec<Pivot>,
    pub signals: Vec<ChanlunSignal>,
    pub current_trend: String,   // "up" | "down" | "consolidation"
}
```

#### 6.2.3 Tauri Commands

```rust
#[tauri::command]
pub async fn analyze_chanlun(state, symbol: String, period: Option<String>) -> Result<ChanlunAnalysis>
// 对指定品种执行缠论分析

#[tauri::command]
pub async fn get_signal_alerts(state, account_id: String) -> Result<Vec<SignalAlert>>
// 获取信号提醒列表

#[tauri::command]
pub async fn create_signal_alert(state, dto: CreateSignalAlertDto) -> Result<SignalAlert>
// 创建信号提醒

#[tauri::command]
pub async fn delete_signal_alert(state, id: String) -> Result<()>
```

### 6.3 前端实现

```
src/
  views/signal-monitor/components/
    ChanlunPanel.vue           -- 新增：缠论分析面板
    SignalAlertList.vue        -- 新增：信号提醒列表
    PivotView.vue              -- 新增：中枢可视化
```

### 6.4 数据库支持

新增迁移 `004_add_signals.sql`：

```sql
CREATE TABLE IF NOT EXISTS signal_alert (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    alert_type TEXT NOT NULL,        -- buy1/sell1/buy2/sell2/buy3/sell3/zg_break/zd_break/divergence
    condition_value REAL,            -- 触发价格
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_triggered INTEGER NOT NULL DEFAULT 0,
    triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_signal_alert_active ON signal_alert(account_id, is_active);
```

---

## 七、模块四：多账户 UI + 多用户支持

### 7.1 多账户 UI 增强

当前账户管理在设置页面中，需要增强为侧边栏顶部的账户切换器。

#### 7.1.1 文件变更

```
src/
  components/
    layout/
      AccountSwitcher.vue          -- 新增：账户切换下拉组件
      Sidebar.vue                  -- 修改：集成账户切换器
  views/settings/
    components/
      AccountManagePanel.vue       -- 新增：账户管理面板（独立于设置）
```

#### 7.1.2 AccountSwitcher 交互

- 侧边栏顶部显示当前账户名称和余额
- 点击展开下拉：显示所有账户列表
- 每个账户显示：名称、余额、本月盈亏
- 底部「管理账户」按钮跳转设置页
- 切换账户后所有数据自动刷新

### 7.2 多用户支持

#### 7.2.1 数据库设计

新增迁移 `005_add_users.sql`：

```sql
CREATE TABLE IF NOT EXISTS user (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    avatar TEXT,                        -- 头像路径（本地文件）
    password_hash TEXT NOT NULL,        -- bcrypt 密码哈希
    is_default INTEGER NOT NULL DEFAULT 0,  -- 默认用户（单用户模式自动登录）
    last_login_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 为现有表添加 user_id 外键（通过迁移脚本 ALTER TABLE）
-- account 表添加 user_id
-- trade_plan 表添加 user_id（通过 account_id 间接关联，可选）
```

#### 7.2.2 登录方案

**简化方案：本地多用户（无需网络）**

- 首次启动：创建默认用户，自动登录
- 设置页面：管理用户（添加/切换/删除）
- 切换用户时重新加载数据
- 支持设置密码（可选，防止他人查看）
- 不实现复杂的注册/找回密码流程

#### 7.2.3 后端实现

```
src-tauri/src/
  services/
    auth_service.rs             -- 新增：用户认证服务
  commands/
    auth.rs                     -- 新增：用户认证 Commands
  models/
    user.rs                     -- 新增：用户模型
  db/
    user_repo.rs                -- 新增：用户数据访问层
```

#### 7.2.4 Tauri Commands

```rust
#[tauri::command]
pub fn get_current_user(state) -> Result<User>

#[tauri::command]
pub fn login(state, username: String, password: String) -> Result<User>

#[tauri::command]
pub fn create_user(state, dto: CreateUserDto) -> Result<User>

#[tauri::command]
pub fn update_user(state, dto: UpdateUserDto) -> Result<User>

#[tauri::command]
pub fn delete_user(state, id: String, password: String) -> Result<()>

#[tauri::command]
pub fn list_users(state) -> Result<Vec<UserBrief>>
// 返回简要用户列表（用于切换用户）

#[tauri::command]
pub fn switch_user(state, user_id: String) -> Result<User>
// 切换当前用户
```

#### 7.2.5 前端实现

```
src/
  views/auth/
    LoginView.vue               -- 新增：登录页
    UserSwitchDialog.vue        -- 新增：用户切换对话框
  stores/
    auth.ts                     -- 新增：认证状态管理
  views/settings/
    components/
      UserManagePanel.vue       -- 新增：用户管理面板
  router/
    index.ts                    -- 修改：添加登录路由守卫
```

---

## 八、完整文件变更清单

### 8.1 新增文件

| 序号 | 文件路径 | 说明 |
|------|----------|------|
| **数据库** | | |
| 1 | `src-tauri/migrations/004_add_signals.sql` | 信号提醒表迁移 |
| 2 | `src-tauri/migrations/005_add_users.sql` | 用户表迁移 |
| **Rust - 自动备份** | | |
| 3 | `src-tauri/src/models/backup.rs` | 备份数据模型 |
| 4 | `src-tauri/src/services/auto_backup_service.rs` | 自动备份服务 |
| 5 | `src-tauri/src/commands/backup.rs` | 备份 Commands |
| **Rust - 行情数据** | | |
| 6 | `src-tauri/src/models/market_data.rs` | 行情数据模型 |
| 7 | `src-tauri/src/services/market_data_service.rs` | 行情数据服务 |
| 8 | `src-tauri/src/commands/market_data.rs` | 行情 Commands |
| **Rust - 缠论信号** | | |
| 9 | `src-tauri/src/models/signal_alert.rs` | 信号模型 |
| 10 | `src-tauri/src/services/chanlun_service.rs` | 缠论分析服务 |
| **Rust - 用户认证** | | |
| 11 | `src-tauri/src/models/user.rs` | 用户模型 |
| 12 | `src-tauri/src/db/user_repo.rs` | 用户数据访问 |
| 13 | `src-tauri/src/services/auth_service.rs` | 认证服务 |
| 14 | `src-tauri/src/commands/auth.rs` | 认证 Commands |
| **前端 - 主题** | | |
| 15 | （修改 main.css） | 暗色主题变量 |
| **前端 - 自动备份** | | |
| 16 | `src/views/settings/components/BackupConfigPanel.vue` | 备份配置面板 |
| 17 | `src/views/settings/components/BackupListPanel.vue` | 备份列表 |
| **前端 - 行情/信号** | | |
| 18 | `src/stores/market.ts` | 行情 Store |
| 19 | `src/stores/auth.ts` | 认证 Store |
| 20 | `src/views/signal-monitor/SignalMonitorView.vue` | 信号监控页 |
| 21 | `src/views/signal-monitor/components/PriceTicker.vue` | 价格条 |
| 22 | `src/views/signal-monitor/components/KlineChart.vue` | K 线图 |
| 23 | `src/views/signal-monitor/components/QuoteCard.vue` | 行情卡片 |
| 24 | `src/views/signal-monitor/components/SymbolSearch.vue` | 品种搜索 |
| 25 | `src/views/signal-monitor/components/ChanlunPanel.vue` | 缠论面板 |
| 26 | `src/views/signal-monitor/components/SignalAlertList.vue` | 信号提醒 |
| 27 | `src/composables/useMarketData.ts` | 行情组合式函数 |
| **前端 - 多账户/多用户** | | |
| 28 | `src/components/layout/AccountSwitcher.vue` | 账户切换器 |
| 29 | `src/views/auth/LoginView.vue` | 登录页 |
| 30 | `src/views/auth/UserSwitchDialog.vue` | 用户切换对话框 |
| 31 | `src/views/settings/components/UserManagePanel.vue` | 用户管理 |

### 8.2 修改文件

| 序号 | 文件路径 | 修改内容 |
|------|----------|----------|
| 1 | `src/assets/css/main.css` | 全面替换为暗色主题变量 |
| 2 | `src/components/ui/**/*.vue` | 所有 UI 组件暗色适配 |
| 3 | `src/components/layout/AppLayout.vue` | 侧边栏暗色 + 集成账户切换 |
| 4 | `src-tauri/src/lib.rs` | 注册新 commands |
| 5 | `src-tauri/src/models/mod.rs` | 导出新模块 |
| 6 | `src-tauri/src/services/mod.rs` | 导出新模块 |
| 7 | `src-tauri/src/commands/mod.rs` | 导出新模块 |
| 8 | `src-tauri/src/db/mod.rs` | 导出 user_repo + 新迁移版本 |
| 9 | `src-tauri/Cargo.toml` | 新增 echarts/vue-echarts 前端依赖 |
| 10 | `src-tauri/tauri.conf.json` | 窗口背景色 |
| 11 | `src/router/index.ts` | 添加信号监控路由 + 登录守卫 |
| 12 | `src/views/settings/SettingsView.vue` | 新增备份配置和用户管理 |
| 13 | `package.json` | 新增 vue-echarts/echarts 依赖 |

---

## 九、实施计划与任务分解

### 总体排期：8 个 Sprint（每个 Sprint 约 1-2 天）

### 优先级排序

```
Sprint 1-2: 暗色主题（视觉基础，所有后续 UI 基于此）
Sprint 3:   自动备份（数据安全优先）
Sprint 4-5: 行情数据 + K 线图（核心新功能）
Sprint 6:   缠论信号检测（依赖行情数据）
Sprint 7:   多账户 UI（数据隔离基础）
Sprint 8:   多用户支持（登录/切换）
```

---

### Sprint 1：暗色主题 - CSS 变量 + Tailwind 配置（P0）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 1.1 | 修改 `main.css` Tailwind 主题变量为暗色方案 | 2h | 无 |
| 1.2 | 修改 `tauri.conf.json` 窗口背景色为 `#0f151d` | 0.5h | 无 |
| 1.3 | 更新所有 UI 基础组件（button/input/card/label/textarea/select/badge）适配暗色 | 4h | 1.1 |
| 1.4 | 更新 toast 组件暗色样式 | 1h | 1.1 |
| 1.5 | 更新 alert-dialog 组件暗色样式 | 1h | 1.1 |

**Sprint 1 交付物：** 所有基础 UI 组件呈现暗色交易终端风格。

---

### Sprint 2：暗色主题 - 页面视图适配（P0）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 2.1 | 适配 `AppLayout.vue`（侧边栏 + 顶栏暗色） | 2h | Sprint 1 |
| 2.2 | 适配 `GlobalSearchDialog.vue` 暗色 | 1h | Sprint 1 |
| 2.3 | 适配 `DashboardView.vue` 暗色（含 Chart.js 图表暗色） | 2h | Sprint 1 |
| 2.4 | 适配 `TradePlanView.vue` 暗色 | 2h | Sprint 1 |
| 2.5 | 适配 `TradeLogView.vue` 暗色 | 1.5h | Sprint 1 |
| 2.6 | 适配 `TradeSummaryView.vue` 暗色 | 1.5h | Sprint 1 |
| 2.7 | 适配 `SettingsView.vue` 暗色 | 1h | Sprint 1 |
| 2.8 | 适配模板相关组件暗色 | 1h | Sprint 1 |

**Sprint 2 交付物：** 整个应用呈现统一的暗色交易终端视觉效果。

---

### Sprint 3：自动备份系统（P0）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 3.1 | 实现 `models/backup.rs` 备份数据模型 | 0.5h | 无 |
| 3.2 | 实现 `services/auto_backup_service.rs` 核心备份逻辑 | 3h | 3.1 |
| 3.3 | 实现 `commands/backup.rs` Tauri Commands | 1.5h | 3.2 |
| 3.4 | 注册备份命令到 `lib.rs` | 0.5h | 3.3 |
| 3.5 | 实现 `BackupConfigPanel.vue` 配置面板 | 3h | 3.3 |
| 3.6 | 实现 `BackupListPanel.vue` 备份列表 | 2h | 3.3 |
| 3.7 | 修改 `SettingsView.vue` 集成备份管理 | 1h | 3.5, 3.6 |
| 3.8 | 实现启动/关闭时自动备份逻辑 | 1h | 3.2 |

**Sprint 3 交付物：** 自动备份系统完整可用，用户可配置备份目录和间隔。

---

### Sprint 4：行情数据服务 + 品种搜索（P1）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 4.1 | 实现 `models/market_data.rs` 行情数据模型 | 0.5h | 无 |
| 4.2 | 实现 `services/market_data_service.rs` Sina API 集成 | 4h | 4.1 |
| 4.3 | 实现 `commands/market_data.rs` Tauri Commands | 1.5h | 4.2 |
| 4.4 | 注册行情命令到 `lib.rs` | 0.5h | 4.3 |
| 4.5 | 新增 `vue-echarts` + `echarts` 前端依赖 | 0.5h | 无 |
| 4.6 | 实现 `stores/market.ts` 行情状态管理 | 2h | 4.3 |
| 4.7 | 实现 `composables/useMarketData.ts` | 1.5h | 4.6 |

**Sprint 4 交付物：** 行情数据后端完整，前端 Store 可获取实时行情和 K 线数据。

---

### Sprint 5：K 线图 + 行情 UI（P1）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 5.1 | 实现 `KlineChart.vue` ECharts K 线图组件 | 4h | Sprint 4 |
| 5.2 | 实现 `PriceTicker.vue` 实时价格显示 | 2h | Sprint 4 |
| 5.3 | 实现 `QuoteCard.vue` 行情卡片 | 1.5h | Sprint 4 |
| 5.4 | 实现 `SymbolSearch.vue` 品种搜索 | 2h | Sprint 4 |
| 5.5 | 实现 `SignalMonitorView.vue` 信号监控页面框架 | 3h | 5.1-5.4 |
| 5.6 | 添加信号监控路由 | 0.5h | 5.5 |

**Sprint 5 交付物：** 信号监控页面完整可用，实时行情和 K 线图展示正常。

---

### Sprint 6：缠论信号检测（P1）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 6.1 | 创建 `004_add_signals.sql` 迁移脚本 | 0.5h | 无 |
| 6.2 | 实现 `models/signal_alert.rs` 信号模型 | 0.5h | 6.1 |
| 6.3 | 实现 `services/chanlun_service.rs` 缠论分析算法 | 6h | Sprint 4 |
| 6.4 | 实现信号检测和提醒逻辑 | 3h | 6.3 |
| 6.5 | 实现 `ChanlunPanel.vue` 缠论分析面板 | 3h | 6.3 |
| 6.6 | 实现 `SignalAlertList.vue` 信号提醒列表 | 2h | 6.4 |
| 6.7 | 在 K 线图上叠加缠论标注（笔/中枢/买卖点） | 3h | 6.3, 5.1 |

**Sprint 6 交付物：** 缠论信号检测功能完整，可在 K 线图上显示分型/笔/中枢/买卖点。

---

### Sprint 7：多账户 UI 增强（P2）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 7.1 | 实现 `AccountSwitcher.vue` 账户切换下拉组件 | 3h | Sprint 2 |
| 7.2 | 修改 `AppLayout.vue` 集成账户切换器到侧边栏 | 2h | 7.1 |
| 7.3 | 实现 `AccountManagePanel.vue` 账户管理增强 | 2h | 7.1 |
| 7.4 | 全局账户切换联动（切换账户后刷新所有数据） | 2h | 7.2 |

**Sprint 7 交付物：** 多账户切换体验流畅，数据隔离正确。

---

### Sprint 8：多用户支持（P2）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 8.1 | 创建 `005_add_users.sql` 迁移脚本 | 0.5h | 无 |
| 8.2 | 实现 `models/user.rs` + `db/user_repo.rs` | 2h | 8.1 |
| 8.3 | 实现 `services/auth_service.rs` 认证服务（含密码哈希） | 3h | 8.2 |
| 8.4 | 实现 `commands/auth.rs` 认证 Commands | 1h | 8.3 |
| 8.5 | 实现 `stores/auth.ts` 前端认证状态管理 | 1.5h | 8.4 |
| 8.6 | 实现 `LoginView.vue` 登录页面 | 3h | 8.5 |
| 8.7 | 实现 `UserSwitchDialog.vue` 用户切换 | 2h | 8.5 |
| 8.8 | 实现 `UserManagePanel.vue` 用户管理 | 2h | 8.5 |
| 8.9 | 路由守卫：未登录跳转登录页 | 1h | 8.6 |
| 8.10 | 首次启动默认用户自动创建 | 1h | 8.3 |
| 8.11 | 版本更新至 v0.6.0 | 0.5h | 全部 |

**Sprint 8 交付物：** 多用户登录完整可用，首次启动自动创建默认用户。

---

## 十、风险与应对

| 风险 | 概率 | 影响 | 应对措施 |
|------|------|------|----------|
| Sina Finance API 不稳定/被封 | 中 | 高 | 实现多数据源降级：Sina → 东方财富 → 手动输入 |
| 缠论算法实现复杂 | 高 | 中 | 分步实现：先笔 → 线段 → 中枢 → 买卖点，每步独立验证 |
| 暗色主题适配工作量大 | 中 | 低 | 基于 CSS 变量统一管理，组件级逐步适配 |
| 备份文件被云同步服务锁定 | 低 | 中 | 备份时先写入临时文件，完成后重命名 |
| 多用户数据隔离不完整 | 中 | 高 | 所有查询强制带 user_id 条件，集成测试覆盖 |

---

## 十一、验收标准

### 11.1 功能验收

- [ ] 整个应用呈现统一的暗色交易终端视觉风格
- [ ] 自动备份可正常工作（启动/定时/关闭三种触发方式）
- [ ] 备份文件可在指定目录正确生成和清理
- [ ] 从备份恢复正常工作
- [ ] Sina Finance 行情数据可正常获取（期货+股票）
- [ ] K 线图正确渲染，支持多周期切换
- [ ] 缠论分型/笔/中枢检测正确
- [ ] 买卖点信号标注清晰
- [ ] 多账户切换流畅，数据隔离正确
- [ ] 多用户登录正常，数据完全隔离

### 11.2 性能验收

- [ ] 暗色主题切换后应用启动无闪白
- [ ] 行情轮询间隔可配置，默认 3 秒，CPU 占用 < 1%
- [ ] K 线图渲染 500 根蜡烛 < 500ms
- [ ] 缠论分析 1000 根 K 线耗时 < 200ms
- [ ] 备份 100MB 数据库耗时 < 5 秒

### 11.3 代码质量

- [ ] Rust 代码通过 `cargo clippy` 检查
- [ ] 前端代码通过 TypeScript 编译无错误
- [ ] 新增文件遵循现有代码风格

---

*文档结束 -- 策盈 TradeMind Phase 6 开发规划 v1.0*
