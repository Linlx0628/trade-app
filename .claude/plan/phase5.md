# Phase 5 开发规划：模板系统 + 导入导出 + 打磨优化

> 项目：策盈 TradeMind（Tauri V2 + Vue 3 + Rust）
> 版本：v0.5.0
> 编写日期：2026-05-14
> 技术负责人审批：待定

---

## 一、Phase 5 总体目标

Phase 5 聚焦三大方向：

1. **交易模板系统** -- 将常用交易计划保存为可复用模板，提升交易计划创建效率
2. **数据导入导出** -- 支持 CSV/Excel 格式的数据导出与导入，保障数据安全与可移植性
3. **打磨优化** -- 完善仪表盘、全局搜索、键盘快捷键和性能优化，提升用户体验到生产级别

---

## 二、技术架构总览

### 2.1 技术栈延续

| 层级 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri V2 | 已集成 fs/dialog/sql/store 插件 |
| 前端框架 | Vue 3 + TypeScript + Pinia | 组合式 API |
| UI 组件 | Radix Vue + Tailwind CSS 4 | 现有 shadcn/ui 风格组件 |
| 后端语言 | Rust | rusqlite + reqwest |
| 数据库 | SQLite (WAL 模式) | 单文件，版本化迁移 |
| 图表库 | Chart.js（新增） | 轻量级，适合仪表盘数据可视化 |

### 2.2 新增依赖

**前端新增：**
```
chart.js          -- 仪表盘图表渲染
@vueuse/integrations  -- useFocusTrap 等增强组合式函数（可选）
```

**Rust 后端新增：**
```toml
# 无新增 crate，CSV 生成使用标准库格式化
# Excel 导出使用简单 XML Spreadsheet 格式（避免引入重依赖）
csv = "1.3"        -- CSV 解析和生成（导入导出）
```

---

## 三、模块一：交易模板系统

### 3.1 需求分析

用户在日常交易中经常重复使用相似的交易计划参数（品种、方向、策略、止损止盈比例等）。模板系统允许用户将一个交易计划保存为模板，后续从模板快速创建新计划，减少重复输入。

### 3.2 数据库设计

新增 `trade_template` 表，通过 `002_add_templates.sql` 迁移脚本创建：

```sql
-- =============================================
-- 交易模板表：保存常用交易计划参数为可复用模板
-- =============================================
CREATE TABLE IF NOT EXISTS trade_template (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,                     -- 关联账户 ID（空字符串表示通用模板）
    name TEXT NOT NULL,                           -- 模板名称
    description TEXT NOT NULL DEFAULT '',         -- 模板描述
    symbol TEXT NOT NULL DEFAULT '',              -- 交易品种
    direction TEXT NOT NULL DEFAULT 'long',       -- 方向: long | short
    market_type TEXT NOT NULL DEFAULT 'futures',  -- 市场类型
    strategy TEXT NOT NULL DEFAULT '',            -- 策略描述
    tags TEXT NOT NULL DEFAULT '[]',              -- 标签 JSON 数组
    -- 以下为可选的价格模板字段（百分比比例方式，避免硬编码价格）
    stop_loss_ratio REAL NOT NULL DEFAULT 0.0,    -- 止损比例（相对入场价的百分比）
    take_profit_ratio REAL NOT NULL DEFAULT 0.0,  -- 止盈比例
    default_lots REAL NOT NULL DEFAULT 0.0,       -- 默认手数
    notes TEXT NOT NULL DEFAULT '',               -- 备注/使用说明
    usage_count INTEGER NOT NULL DEFAULT 0,       -- 使用次数统计
    sort_order INTEGER NOT NULL DEFAULT 0,        -- 排序权重
    is_pinned INTEGER NOT NULL DEFAULT 0,         -- 是否置顶
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_template_account ON trade_template(account_id);
CREATE INDEX IF NOT EXISTS idx_template_pinned ON trade_template(is_pinned);
```

**设计要点：**
- 模板使用比例（`stop_loss_ratio`/`take_profit_ratio`）而非固定价格，因为每次交易的入场价不同
- `account_id` 支持空字符串（表示全局通用模板），也支持绑定特定账户
- `usage_count` 记录使用频率，可用于智能排序
- `is_pinned` 支持置顶常用模板

### 3.3 后端实现

#### 3.3.1 文件结构

```
src-tauri/src/
  models/
    trade_template.rs      -- 新增：模板数据模型 + DTO
    mod.rs                 -- 修改：导出 trade_template 模块
  db/
    trade_template_repo.rs -- 新增：模板数据访问层
    mod.rs                 -- 修改：导出 trade_template_repo 模块
  services/
    trade_template_service.rs -- 新增：模板业务逻辑层
    mod.rs                    -- 修改：导出 TradeTemplateService
  commands/
    trade_template.rs      -- 新增：模板 Tauri Commands
    mod.rs                  -- 修改：导出 trade_template 模块
  lib.rs                   -- 修改：注册新 commands
migrations/
  002_add_templates.sql    -- 新增：数据库迁移脚本
```

#### 3.3.2 Rust 模型定义（`models/trade_template.rs`）

```rust
// 核心结构体
pub struct TradeTemplate {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub direction: String,
    pub market_type: String,
    pub strategy: String,
    pub tags: String,           // JSON 数组
    pub stop_loss_ratio: f64,
    pub take_profit_ratio: f64,
    pub default_lots: f64,
    pub notes: String,
    pub usage_count: i64,
    pub sort_order: i64,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

// 创建 DTO
pub struct CreateTradeTemplateDto {
    pub account_id: String,
    pub name: String,
    pub description: Option<String>,
    pub symbol: Option<String>,
    pub direction: Option<String>,
    pub market_type: Option<String>,
    pub strategy: Option<String>,
    pub tags: Option<Vec<String>>,
    pub stop_loss_ratio: Option<f64>,
    pub take_profit_ratio: Option<f64>,
    pub default_lots: Option<f64>,
    pub notes: Option<String>,
}

// 更新 DTO
pub struct UpdateTradeTemplateDto {
    pub id: String,
    // ... 所有字段均为 Option
}

// 从计划创建模板 DTO
pub struct CreateTemplateFromPlanDto {
    pub plan_id: String,
    pub template_name: String,
    pub template_description: Option<String>,
}

// 从模板创建计划 DTO
pub struct CreatePlanFromTemplateDto {
    pub template_id: String,
    pub account_id: String,
    pub entry_price: f64,        // 必须提供入场价，用于计算止损止盈
    pub planned_at: Option<String>,
    pub actual_lots: Option<f64>, // 可覆盖默认手数
}
```

#### 3.3.3 Tauri Commands（`commands/trade_template.rs`）

```rust
#[tauri::command]
pub fn get_trade_templates(state, account_id: String) -> Result<Vec<TradeTemplate>>

#[tauri::command]
pub fn get_trade_template(state, id: String) -> Result<TradeTemplate>

#[tauri::command]
pub fn create_trade_template(state, dto: CreateTradeTemplateDto) -> Result<TradeTemplate>

#[tauri::command]
pub fn update_trade_template(state, dto: UpdateTradeTemplateDto) -> Result<TradeTemplate>

#[tauri::command]
pub fn delete_trade_template(state, id: String) -> Result<()>

#[tauri::command]
pub fn create_template_from_plan(state, dto: CreateTemplateFromPlanDto) -> Result<TradeTemplate>
// 从已有计划保存为模板：提取计划的品种、方向、策略等，计算止损止盈比例

#[tauri::command]
pub fn create_plan_from_template(state, dto: CreatePlanFromTemplateDto) -> Result<TradePlan>
// 从模板创建新计划：根据入场价和模板中的比例计算止损止盈价格
```

### 3.4 前端实现

#### 3.4.1 文件结构

```
src/
  types/
    common.ts              -- 修改：新增 TradeTemplate 类型定义
  stores/
    trade-template.ts      -- 新增：模板状态管理
  views/
    trade-plan/
      TradePlanView.vue    -- 修改：集成模板选择器
      components/
        TemplateSelector.vue     -- 新增：模板选择下拉面板
        TemplateSaveDialog.vue   -- 新增：保存为模板对话框
        TemplateManagePanel.vue  -- 新增：模板管理面板（在设置页或弹窗中）
  router/
    index.ts               -- 修改：新增模板管理路由（可选）
```

#### 3.4.2 前端 Store（`stores/trade-template.ts`）

```typescript
// 核心 state
{
  templates: TradeTemplate[]
  loading: boolean
  error: string | null
}

// 核心 actions
fetchTemplates(accountId: string)
createTemplate(dto)
updateTemplate(dto)
deleteTemplate(id)
createFromPlan(planId, name, description)
createFromTemplate(templateId, accountId, entryPrice, lots?)
```

#### 3.4.3 UI 交互流程

**从模板创建计划：**
1. 在交易计划页面点击「从模板创建」按钮
2. 弹出模板选择器面板（显示所有可用模板，按使用频率排序，置顶优先）
3. 选择模板后，自动填充品种、方向、策略等字段
4. 用户输入入场价格，系统自动计算止损止盈（基于模板中的比例）
5. 用户确认后创建计划

**保存计划为模板：**
1. 在已有交易计划的操作菜单中选择「保存为模板」
2. 弹出对话框，显示将保存的字段预览
3. 用户填写模板名称和描述
4. 系统自动计算止损止盈比例并保存

**模板管理：**
1. 在设置页面新增「模板管理」标签页
2. 列表展示所有模板，支持编辑、删除、置顶、排序
3. 显示每个模板的使用次数统计

---

## 四、模块二：数据导入导出

### 4.1 需求分析

用户需要将交易数据导出为通用格式（CSV/Excel）进行离线分析或备份，同时支持从外部数据源导入交易记录。系统还需提供完整的数据备份与恢复功能。

### 4.2 数据库设计

无需新增数据库表。导入导出是对现有数据的序列化/反序列化操作。

### 4.3 后端实现

#### 4.3.1 文件结构

```
src-tauri/src/
  services/
    export_service.rs      -- 新增：导出服务（CSV/Excel 生成）
    import_service.rs      -- 新增：导入服务（CSV 解析 + 数据验证）
    backup_service.rs      -- 新增：备份恢复服务
    mod.rs                 -- 修改：导出新服务
  commands/
    data_io.rs             -- 新增：导入导出 Tauri Commands
    mod.rs                 -- 修改：导出 data_io 模块
  lib.rs                   -- 修改：注册新 commands
```

#### 4.3.2 导出功能设计

**CSV 导出格式（交易日志）：**
```
品种代码,品种名称,方向,市场类型,入场价格,出场价格,止损价格,手数,盈亏金额,盈亏点数,手续费,状态,入场时间,出场时间,标签,备注,情绪(前),情绪(后),信心指数
```

**CSV 导出格式（交易计划）：**
```
品种代码,品种名称,方向,市场类型,入场价格,止损价格,止盈价格,手数,状态,策略,标签,备注,计划时间
```

**Excel 导出：** 使用 CSV 格式写入 `.xlsx` 兼容的 XML Spreadsheet 格式，或直接导出 `.csv` 让用户在 Excel 中打开。

**完整数据备份：** 将 SQLite 数据库文件直接复制到用户指定目录，同时导出一个包含元信息的 JSON 清单文件。

#### 4.3.3 核心数据结构

```rust
// 导出请求
pub struct ExportRequest {
    pub data_type: String,         // "trade_logs" | "trade_plans" | "trade_summaries" | "all"
    pub account_id: String,
    pub format: String,            // "csv" | "excel"
    pub date_range: Option<DateRange>,
    pub file_path: String,         // 用户选择的保存路径
}

pub struct DateRange {
    pub start: String,
    pub end: String,
}

// 导入请求
pub struct ImportRequest {
    pub file_path: String,
    pub data_type: String,         // "trade_logs" | "trade_plans"
    pub account_id: String,        // 导入到哪个账户
    pub conflict_strategy: String, // "skip" | "overwrite" | "rename"
}

// 导入结果
pub struct ImportResult {
    pub total_rows: i64,
    pub imported: i64,
    pub skipped: i64,
    pub errors: Vec<ImportError>,
}

pub struct ImportError {
    pub row: i64,
    pub field: String,
    pub message: String,
}

// 备份请求
pub struct BackupRequest {
    pub backup_path: String,       // 备份目录路径
    pub include_attachments: bool, // 是否包含附件（图片等）
}

// 备份元信息
pub struct BackupMeta {
    pub version: String,
    pub created_at: String,
    pub account_count: i64,
    pub plan_count: i64,
    pub log_count: i64,
    pub summary_count: i64,
}
```

#### 4.3.4 Tauri Commands

```rust
// 导出相关
#[tauri::command]
pub async fn export_data(state, request: ExportRequest) -> Result<String>
// 使用 tauri_plugin_dialog 的文件选择器让用户选路径
// 调用 export_service 生成文件

#[tauri::command]
pub async fn export_csv(state, account_id: String, data_type: String, file_path: String) -> Result<String>

// 导入相关
#[tauri::command]
pub async fn import_data(state, request: ImportRequest) -> Result<ImportResult>
// 解析 CSV，逐行验证，批量写入数据库

#[tauri::command]
pub async fn preview_import(state, file_path: String, data_type: String) -> Result<ImportPreview>
// 预览导入数据（前 10 行），用户确认后再执行导入

// 备份恢复
#[tauri::command]
pub async fn create_backup(state, request: BackupRequest) -> Result<BackupMeta>
// 复制数据库文件 + 生成元信息 JSON

#[tauri::command]
pub async fn restore_backup(state, backup_path: String) -> Result<()>
// 验证备份完整性，替换当前数据库

#[tauri::command]
pub async fn get_backup_info(state, backup_path: String) -> Result<BackupMeta>
// 读取备份元信息，供用户确认
```

### 4.4 前端实现

#### 4.4.1 文件结构

```
src/
  views/
    settings/
      SettingsView.vue          -- 修改：新增「数据管理」区域
      components/
        DataExportPanel.vue     -- 新增：导出操作面板
        DataImportPanel.vue     -- 新增：导入操作面板
        BackupRestorePanel.vue  -- 新增：备份恢复面板
        ImportPreviewDialog.vue -- 新增：导入预览对话框
  composables/
    useExport.ts                -- 新增：导出组合式函数
    useImport.ts                -- 新增：导入组合式函数
```

#### 4.4.2 UI 交互流程

**数据导出：**
1. 设置页面中新增「数据管理」区块
2. 用户选择导出类型（交易日志/计划/总结/全部）
3. 可选日期范围筛选
4. 选择格式（CSV / Excel）
5. 点击导出，系统调用 Tauri 文件选择对话框选择保存路径
6. 生成文件，显示成功提示

**数据导入：**
1. 用户点击「导入数据」按钮
2. 选择文件（CSV）
3. 系统解析文件，弹出预览对话框显示前 10 行数据
4. 用户选择目标账户和冲突处理策略
5. 确认导入，显示进度和结果（成功数/跳过数/错误数）

**备份恢复：**
1. 「备份」按钮：选择备份目录，一键复制数据库
2. 「恢复」按钮：选择备份文件，显示备份信息，确认后恢复

---

## 五、模块三：打磨优化

### 5.1 仪表盘完善

#### 5.1.1 功能设计

当前仪表盘为占位页面（"开发中"），需要实现完整的交易数据统计概览。

#### 5.1.2 仪表盘内容

**顶部统计卡片行：**
- 账户净值（当前余额）
- 今日盈亏（当日已平仓交易的净盈亏合计）
- 总盈亏（全部历史净盈亏）
- 胜率（总盈利笔数 / 总平仓笔数）
- 本月交易笔数

**图表区域：**
- **盈亏趋势图**（折线图）：近 30 天每日净盈亏趋势
- **盈亏分布图**（柱状图）：按交易品种分组的盈亏分布
- **胜率趋势图**（面积图）：滚动 20 笔交易的胜率变化

**快捷操作区：**
- 最近 5 条交易日志（快速查看）
- 最近 5 条未完成的交易计划（快速操作）
- 快速创建按钮（新建计划、新建日志）

#### 5.1.3 后端支持

```
src-tauri/src/
  services/
    dashboard_service.rs   -- 新增：仪表盘统计数据聚合服务
  commands/
    dashboard.rs           -- 新增：仪表盘 Tauri Commands
```

```rust
// 仪表盘统计数据
pub struct DashboardStats {
    pub account_id: String,
    pub balance: f64,
    pub today_pnl: f64,
    pub total_pnl: f64,
    pub total_trades: i64,
    pub win_trades: i64,
    pub loss_trades: i64,
    pub win_rate: f64,
    pub month_trades: i64,
    pub month_pnl: f64,
    pub open_positions: i64,       // 当前持仓中交易数
    pub pending_plans: i64,        // 待执行的计划数
}

// 盈亏趋势数据
pub struct PnlTrend {
    pub date: String,
    pub pnl: f64,
    pub cumulative_pnl: f64,       // 累计盈亏
}

// 品种盈亏分布
pub struct SymbolPnl {
    pub symbol: String,
    pub name: String,
    pub trade_count: i64,
    pub total_pnl: f64,
    pub win_rate: f64,
}

#[tauri::command]
pub fn get_dashboard_stats(state, account_id: String) -> Result<DashboardStats>

#[tauri::command]
pub fn get_pnl_trend(state, account_id: String, days: Option<i64>) -> Result<Vec<PnlTrend>>

#[tauri::command]
pub fn get_symbol_pnl(state, account_id: String) -> Result<Vec<SymbolPnl>>

#[tauri::command]
pub fn get_recent_logs(state, account_id: String, limit: Option<i64>) -> Result<Vec<TradeLog>>

#[tauri::command]
pub fn get_pending_plans(state, account_id: String, limit: Option<i64>) -> Result<Vec<TradePlan>>
```

#### 5.1.4 前端实现

```
src/
  views/
    dashboard/
      DashboardView.vue            -- 重写：完整仪表盘页面
      components/
        StatsCards.vue             -- 新增：统计卡片行
        PnlTrendChart.vue          -- 新增：盈亏趋势图
        SymbolPnlChart.vue         -- 新增：品种盈亏分布图
        RecentTradesWidget.vue     -- 新增：最近交易列表
        PendingPlansWidget.vue     -- 新增：待执行计划列表
  stores/
    dashboard.ts                   -- 新增：仪表盘状态管理
```

**Chart.js 集成方式：**
- 使用 `chart.js` + `vue-chartjs` 封装（或直接使用 Canvas API + Chart.js）
- 主题色与现有设计系统一致（profit 绿色、loss 红色、muted 灰色）
- 图表响应式，跟随容器尺寸自适应

### 5.2 全局搜索

#### 5.2.1 功能设计

通过 `Cmd/Ctrl + K` 快捷键唤起全局搜索面板，支持搜索：
- 交易计划（按品种、策略、备注搜索）
- 交易日志（按品种、备注搜索）
- 交易总结（按市场观点、教训搜索）
- 模板（按名称、描述搜索）

#### 5.2.2 后端支持

```
src-tauri/src/
  services/
    search_service.rs     -- 新增：全局搜索服务
  commands/
    search.rs             -- 新增：搜索 Tauri Commands
```

```rust
pub struct SearchResult {
    pub id: String,
    pub item_type: String,    // "plan" | "log" | "summary" | "template"
    pub title: String,
    pub subtitle: String,
    pub symbol: Option<String>,
    pub date: String,
    pub match_field: String,  // 匹配的字段名
}

#[tauri::command]
pub fn global_search(state, account_id: String, query: String, limit: Option<i64>) -> Result<Vec<SearchResult>>
// 在所有表中执行 LIKE 查询，合并结果，按相关性排序
```

#### 5.2.3 前端实现

```
src/
  components/
    layout/
      AppLayout.vue             -- 修改：添加搜索快捷键监听
      GlobalSearchDialog.vue    -- 新增：全局搜索面板
  composables/
    useSearch.ts                -- 新增：搜索组合式函数
```

**搜索面板 UI：**
- 模态对话框，半透明背景遮罩
- 顶部搜索输入框（自动聚焦）
- 分类展示搜索结果（计划/日志/总结/模板）
- 键盘导航（上下箭头选择，Enter 跳转，Esc 关闭）
- 搜索防抖（300ms）

### 5.3 键盘快捷键

#### 5.3.1 快捷键列表

| 快捷键 | 功能 | 适用场景 |
|--------|------|----------|
| `Cmd/Ctrl + K` | 全局搜索 | 全局 |
| `Cmd/Ctrl + N` | 新建交易计划 | 全局 |
| `Cmd/Ctrl + E` | 导出数据 | 全局 |
| `Cmd/Ctrl + ,` | 打开设置 | 全局 |
| `Cmd/Ctrl + 1` | 切换到仪表盘 | 全局 |
| `Cmd/Ctrl + 2` | 切换到交易计划 | 全局 |
| `Cmd/Ctrl + 3` | 切换到交易日志 | 全局 |
| `Cmd/Ctrl + 4` | 切换到交易总结 | 全局 |
| `Esc` | 关闭弹窗/取消编辑 | 弹窗/表单 |
| `Enter` | 确认操作 | 表单 |

#### 5.3.2 实现方式

```
src/
  composables/
    useKeyboardShortcuts.ts   -- 新增：全局快捷键组合式函数
  components/
    layout/
      AppLayout.vue           -- 修改：注册全局快捷键
      ShortcutHelp.vue        -- 新增：快捷键帮助面板
```

使用 `@vueuse/core` 的 `useEventListener` 监听 `keydown` 事件，在 `AppLayout.vue` 中统一注册。

### 5.4 性能优化

#### 5.4.1 列表虚拟滚动

当交易日志或计划数据量超过 100 条时，使用虚拟滚动减少 DOM 节点数量。

**实现方式：** 自行实现轻量级虚拟滚动（基于 `IntersectionObserver` + 动态高度计算），避免引入重型虚拟滚动库。

#### 5.4.2 数据加载优化

- 交易计划/日志列表支持**分页加载**（每页 20 条，滚动到底部自动加载下一页）
- 仪表盘统计数据增加**缓存机制**（5 分钟内复用缓存数据）
- 图片列表使用**懒加载**（IntersectionObserver）

**后端分页支持：**
```rust
pub struct Pagination {
    pub page: i64,       // 页码（从 1 开始）
    pub page_size: i64,  // 每页数量
}

pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
```

#### 5.4.3 Rust 后端优化

- 批量查询使用 Prepared Statements（预编译 SQL）
- 对高频查询添加复合索引
- WAL 模式下读写分离配置优化

**新增数据库索引（迁移 003）：**
```sql
CREATE INDEX IF NOT EXISTS idx_trade_log_symbol_date ON trade_log(symbol, entry_time);
CREATE INDEX IF NOT EXISTS idx_trade_plan_created ON trade_plan(created_at);
CREATE INDEX IF NOT EXISTS idx_trade_log_pnl ON trade_log(pnl);
```

---

## 六、完整文件变更清单

### 6.1 新增文件

| 序号 | 文件路径 | 说明 |
|------|----------|------|
| **数据库** | | |
| 1 | `src-tauri/migrations/002_add_templates.sql` | 模板表迁移脚本 |
| 2 | `src-tauri/migrations/003_add_indexes.sql` | 性能优化索引 |
| **Rust 后端 - 模板** | | |
| 3 | `src-tauri/src/models/trade_template.rs` | 模板数据模型 |
| 4 | `src-tauri/src/db/trade_template_repo.rs` | 模板数据访问层 |
| 5 | `src-tauri/src/services/trade_template_service.rs` | 模板业务逻辑 |
| 6 | `src-tauri/src/commands/trade_template.rs` | 模板 Tauri Commands |
| **Rust 后端 - 导入导出** | | |
| 7 | `src-tauri/src/services/export_service.rs` | 导出服务 |
| 8 | `src-tauri/src/services/import_service.rs` | 导入服务 |
| 9 | `src-tauri/src/services/backup_service.rs` | 备份恢复服务 |
| 10 | `src-tauri/src/commands/data_io.rs` | 导入导出 Commands |
| **Rust 后端 - 仪表盘/搜索** | | |
| 11 | `src-tauri/src/services/dashboard_service.rs` | 仪表盘统计服务 |
| 12 | `src-tauri/src/commands/dashboard.rs` | 仪表盘 Commands |
| 13 | `src-tauri/src/services/search_service.rs` | 全局搜索服务 |
| 14 | `src-tauri/src/commands/search.rs` | 搜索 Commands |
| **前端 - 模板** | | |
| 15 | `src/stores/trade-template.ts` | 模板 Store |
| 16 | `src/views/trade-plan/components/TemplateSelector.vue` | 模板选择器 |
| 17 | `src/views/trade-plan/components/TemplateSaveDialog.vue` | 保存模板对话框 |
| **前端 - 导入导出** | | |
| 18 | `src/views/settings/components/DataExportPanel.vue` | 导出面板 |
| 19 | `src/views/settings/components/DataImportPanel.vue` | 导入面板 |
| 20 | `src/views/settings/components/BackupRestorePanel.vue` | 备份恢复面板 |
| 21 | `src/views/settings/components/ImportPreviewDialog.vue` | 导入预览 |
| 22 | `src/composables/useExport.ts` | 导出组合式函数 |
| 23 | `src/composables/useImport.ts` | 导入组合式函数 |
| **前端 - 仪表盘** | | |
| 24 | `src/stores/dashboard.ts` | 仪表盘 Store |
| 25 | `src/views/dashboard/components/StatsCards.vue` | 统计卡片 |
| 26 | `src/views/dashboard/components/PnlTrendChart.vue` | 盈亏趋势图 |
| 27 | `src/views/dashboard/components/SymbolPnlChart.vue` | 品种分布图 |
| 28 | `src/views/dashboard/components/RecentTradesWidget.vue` | 最近交易 |
| 29 | `src/views/dashboard/components/PendingPlansWidget.vue` | 待执行计划 |
| **前端 - 搜索/快捷键** | | |
| 30 | `src/components/layout/GlobalSearchDialog.vue` | 全局搜索面板 |
| 31 | `src/components/layout/ShortcutHelp.vue` | 快捷键帮助 |
| 32 | `src/composables/useSearch.ts` | 搜索组合式函数 |
| 33 | `src/composables/useKeyboardShortcuts.ts` | 快捷键组合式函数 |

### 6.2 修改文件

| 序号 | 文件路径 | 修改内容 |
|------|----------|----------|
| **Rust** | | |
| 1 | `src-tauri/src/models/mod.rs` | 新增 `pub mod trade_template` |
| 2 | `src-tauri/src/db/mod.rs` | 新增 `pub mod trade_template_repo` + 迁移版本 |
| 3 | `src-tauri/src/services/mod.rs` | 新增 4 个 service 模块导出 |
| 4 | `src-tauri/src/commands/mod.rs` | 新增 4 个 command 模块导出 |
| 5 | `src-tauri/src/lib.rs` | 注册所有新 commands 到 invoke_handler |
| 6 | `src-tauri/Cargo.toml` | 新增 `csv` 依赖 |
| **前端** | | |
| 7 | `src/types/common.ts` | 新增 TradeTemplate、DashboardStats 等类型 |
| 8 | `src/views/dashboard/DashboardView.vue` | 重写为完整仪表盘 |
| 9 | `src/views/trade-plan/TradePlanView.vue` | 集成模板选择和保存 |
| 10 | `src/views/settings/SettingsView.vue` | 新增数据管理区块 |
| 11 | `src/components/layout/AppLayout.vue` | 添加搜索快捷键 + 全局搜索组件 |
| 12 | `src/router/index.ts` | 可选新增模板管理路由 |
| 13 | `package.json` | 新增 `chart.js` 依赖 |

---

## 七、实施计划与任务分解

### 7.1 总体排期：6 个迭代周期（每个周期约 1-2 天）

### 7.2 任务优先级排序

按**核心价值**和**依赖关系**排列：

---

#### Sprint 1：数据库基础 + 模板后端（优先级 P0）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 1.1 | 创建 `002_add_templates.sql` 迁移脚本 | 0.5h | 无 |
| 1.2 | 创建 `003_add_indexes.sql` 迁移脚本 | 0.5h | 无 |
| 1.3 | 实现 `models/trade_template.rs` 数据模型 | 1h | 1.1 |
| 1.4 | 实现 `db/trade_template_repo.rs` 数据访问层 | 2h | 1.3 |
| 1.5 | 实现 `services/trade_template_service.rs` 业务逻辑 | 2h | 1.4 |
| 1.6 | 实现 `commands/trade_template.rs` Tauri Commands | 1h | 1.5 |
| 1.7 | 更新 `lib.rs`、`mod.rs` 注册新模块 | 0.5h | 1.6 |

**Sprint 1 交付物：** 模板 CRUD 后端完整可用，通过 Tauri DevTools 可测试所有 API。

---

#### Sprint 2：模板前端 + 设置页数据管理（优先级 P0）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 2.1 | 新增 `types/common.ts` 模板类型定义 | 0.5h | 无 |
| 2.2 | 实现 `stores/trade-template.ts` | 1.5h | 2.1 |
| 2.3 | 实现 `TemplateSelector.vue` 模板选择器 | 3h | 2.2 |
| 2.4 | 实现 `TemplateSaveDialog.vue` 保存模板对话框 | 2h | 2.2 |
| 2.5 | 修改 `TradePlanView.vue` 集成模板功能 | 2h | 2.3, 2.4 |
| 2.6 | 修改 `SettingsView.vue` 新增模板管理区域 | 2h | 2.2 |

**Sprint 2 交付物：** 模板系统前后端联调完成，用户可以创建、使用、管理模板。

---

#### Sprint 3：导入导出后端 + 前端（优先级 P1）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 3.1 | 新增 `csv` 依赖到 Cargo.toml | 0.5h | 无 |
| 3.2 | 实现 `services/export_service.rs` | 3h | 3.1 |
| 3.3 | 实现 `services/import_service.rs` | 3h | 3.1 |
| 3.4 | 实现 `services/backup_service.rs` | 2h | 无 |
| 3.5 | 实现 `commands/data_io.rs` | 1.5h | 3.2, 3.3, 3.4 |
| 3.6 | 实现前端导入导出面板组件 | 3h | 3.5 |
| 3.7 | 修改 `SettingsView.vue` 集成数据管理 | 2h | 3.6 |

**Sprint 3 交付物：** 数据导入导出功能完整可用，支持 CSV 导出/导入和数据库备份恢复。

---

#### Sprint 4：仪表盘完善（优先级 P1）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 4.1 | 新增 `chart.js` 前端依赖 | 0.5h | 无 |
| 4.2 | 实现 `services/dashboard_service.rs` | 3h | Sprint 1 索引 |
| 4.3 | 实现 `commands/dashboard.rs` | 1h | 4.2 |
| 4.4 | 实现 `stores/dashboard.ts` | 1.5h | 4.3 |
| 4.5 | 实现 `StatsCards.vue` 统计卡片 | 2h | 4.4 |
| 4.6 | 实现 `PnlTrendChart.vue` 盈亏趋势图 | 3h | 4.4, 4.1 |
| 4.7 | 实现 `SymbolPnlChart.vue` 品种分布图 | 2h | 4.4, 4.1 |
| 4.8 | 实现 `RecentTradesWidget.vue` + `PendingPlansWidget.vue` | 2h | 4.4 |
| 4.9 | 重写 `DashboardView.vue` 组装所有组件 | 2h | 4.5-4.8 |

**Sprint 4 交付物：** 仪表盘页面完整可用，包含统计卡片、图表和快捷操作。

---

#### Sprint 5：全局搜索 + 键盘快捷键（优先级 P2）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 5.1 | 实现 `services/search_service.rs` | 2h | Sprint 1 索引 |
| 5.2 | 实现 `commands/search.rs` | 1h | 5.1 |
| 5.3 | 实现 `composables/useSearch.ts` | 1.5h | 5.2 |
| 5.4 | 实现 `GlobalSearchDialog.vue` | 3h | 5.3 |
| 5.5 | 实现 `composables/useKeyboardShortcuts.ts` | 2h | 无 |
| 5.6 | 实现 `ShortcutHelp.vue` 快捷键帮助 | 1h | 5.5 |
| 5.7 | 修改 `AppLayout.vue` 集成搜索和快捷键 | 2h | 5.4, 5.5 |

**Sprint 5 交付物：** `Cmd+K` 全局搜索可用，所有键盘快捷键生效。

---

#### Sprint 6：性能优化 + 集成测试（优先级 P2）

| 任务 | 描述 | 预计耗时 | 依赖 |
|------|------|----------|------|
| 6.1 | 后端分页查询实现 | 2h | 无 |
| 6.2 | 前端分页/虚拟滚动 | 3h | 6.1 |
| 6.3 | 数据缓存策略（仪表盘统计数据） | 1.5h | Sprint 4 |
| 6.4 | 图片懒加载优化 | 1h | 无 |
| 6.5 | 全功能集成测试 | 3h | Sprint 1-5 |
| 6.6 | UI 细节打磨和 Bug 修复 | 3h | Sprint 1-5 |
| 6.7 | 应用版本更新到 v0.5.0 | 0.5h | 6.5 |

**Sprint 6 交付物：** Phase 5 全部功能完整、性能达标、Bug 清零。

---

## 八、风险与应对

| 风险 | 概率 | 影响 | 应对措施 |
|------|------|------|----------|
| CSV 导入格式不兼容 | 中 | 中 | 提供标准模板下载，导入时做严格的字段校验和错误提示 |
| Chart.js 与 Tailwind CSS 4 冲突 | 低 | 低 | 使用 CSS 变量传递主题色给 Chart.js，避免样式冲突 |
| 模板比例计算精度问题 | 低 | 中 | 使用 f64 双精度，前端展示时统一格式化到小数点后两位 |
| 大数据量下仪表盘加载慢 | 中 | 中 | 后端聚合查询 + 前端缓存，避免传输大量明细数据 |
| 备份恢复时数据库锁定 | 低 | 高 | 恢复时提示用户关闭其他操作，使用事务保证原子性 |

---

## 九、验收标准

### 9.1 功能验收

- [ ] 模板 CRUD 完整可用（创建、编辑、删除、置顶）
- [ ] 从模板创建计划流程顺畅，止损止盈自动计算正确
- [ ] 从计划保存为模板，比例计算正确
- [ ] CSV 导出包含完整字段，Excel 可正确打开
- [ ] CSV 导入支持预览，错误数据有明确提示
- [ ] 数据库备份和恢复正常工作
- [ ] 仪表盘显示正确的统计数据和图表
- [ ] 全局搜索返回准确结果，支持跨类型搜索
- [ ] 所有键盘快捷键在 macOS 和 Windows 上均可正常使用

### 9.2 性能验收

- [ ] 仪表盘页面加载时间 < 1 秒（数据量 < 1000 条）
- [ ] 全局搜索响应时间 < 200ms（数据量 < 10000 条）
- [ ] 导出 1000 条记录耗时 < 3 秒
- [ ] 导入 500 条记录耗时 < 5 秒
- [ ] 列表滚动流畅，无卡顿

### 9.3 代码质量

- [ ] Rust 代码通过 `cargo clippy` 检查
- [ ] 前端代码通过 TypeScript 编译无错误
- [ ] 新增文件遵循现有代码风格和命名规范
- [ ] 数据库迁移脚本可幂等执行

---

## 十、后续展望（Phase 6+）

Phase 5 完成后，可考虑以下方向：

1. **多账户对比分析** -- 不同账户间的业绩对比
2. **交易策略回测** -- 基于历史日志的简单策略回测
3. **社区模板共享** -- 导出/导入模板的标准化格式
4. **移动端适配** -- 响应式布局优化或 Tauri Mobile 支持
5. **国际化（i18n）** -- 多语言支持
6. **自动同步** -- 云端数据同步（WebDAV/S3）

---

*文档结束 -- 策盈 TradeMind Phase 5 开发规划 v1.0*
