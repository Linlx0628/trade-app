-- 交易管理系统数据库初始化脚本
-- 版本: 001
-- 说明: 创建基础数据表（账户、交易计划、交易日志、交易总结）

-- =============================================
-- 账户表：存储交易账户信息
-- =============================================
CREATE TABLE IF NOT EXISTS account (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,                          -- 账户名称
    balance REAL NOT NULL DEFAULT 0.0,           -- 账户余额
    risk_ratio REAL NOT NULL DEFAULT 0.02,       -- 风险比例（如 0.02 表示 2%）
    point_value REAL NOT NULL DEFAULT 1.0,       -- 每点价值（期货合约乘数）
    market_type TEXT NOT NULL DEFAULT 'futures',  -- 市场类型: futures | stock
    description TEXT NOT NULL DEFAULT '',         -- 账户描述
    is_active INTEGER NOT NULL DEFAULT 1,        -- 是否激活: 1=是, 0=否
    created_at TEXT NOT NULL,                     -- 创建时间 (RFC3339)
    updated_at TEXT NOT NULL                      -- 更新时间 (RFC3339)
);

-- =============================================
-- 交易计划表：存储每笔交易的计划
-- =============================================
CREATE TABLE IF NOT EXISTS trade_plan (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,                    -- 关联账户 ID
    symbol TEXT NOT NULL,                         -- 交易品种/代码
    name TEXT NOT NULL DEFAULT '',                -- 品种名称
    direction TEXT NOT NULL DEFAULT 'long',       -- 交易方向: long | short
    entry_price REAL NOT NULL DEFAULT 0.0,        -- 入场价格
    stop_loss REAL NOT NULL DEFAULT 0.0,          -- 止损价格
    take_profit REAL NOT NULL DEFAULT 0.0,        -- 止盈价格
    lots REAL NOT NULL DEFAULT 0.0,               -- 手数/数量
    market_type TEXT NOT NULL DEFAULT 'futures',   -- 市场类型: futures | stock
    status TEXT NOT NULL DEFAULT 'planned',        -- 状态: planned | executing | completed | cancelled
    strategy TEXT NOT NULL DEFAULT '',             -- 交易策略描述
    tags TEXT NOT NULL DEFAULT '[]',               -- 标签 (JSON 数组字符串)
    notes TEXT NOT NULL DEFAULT '',                -- 备注
    images TEXT NOT NULL DEFAULT '[]',             -- 图片列表 (JSON 数组字符串)
    planned_at TEXT NOT NULL DEFAULT '',           -- 计划交易时间
    created_at TEXT NOT NULL,                      -- 创建时间 (RFC3339)
    updated_at TEXT NOT NULL,                      -- 更新时间 (RFC3339)
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);

-- =============================================
-- 交易日志表：记录交易执行和结果
-- =============================================
CREATE TABLE IF NOT EXISTS trade_log (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,                    -- 关联账户 ID
    plan_id TEXT,                                 -- 关联交易计划 ID（可为空）
    symbol TEXT NOT NULL,                         -- 交易品种/代码
    name TEXT NOT NULL DEFAULT '',                -- 品种名称
    direction TEXT NOT NULL DEFAULT 'long',       -- 交易方向: long | short
    entry_price REAL NOT NULL DEFAULT 0.0,        -- 实际入场价格
    exit_price REAL NOT NULL DEFAULT 0.0,         -- 实际出场价格
    stop_loss REAL NOT NULL DEFAULT 0.0,          -- 实际止损价格
    lots REAL NOT NULL DEFAULT 0.0,               -- 实际手数
    pnl REAL NOT NULL DEFAULT 0.0,                -- 盈亏金额
    pnl_points REAL NOT NULL DEFAULT 0.0,         -- 盈亏点数
    commission REAL NOT NULL DEFAULT 0.0,         -- 手续费
    market_type TEXT NOT NULL DEFAULT 'futures',   -- 市场类型
    status TEXT NOT NULL DEFAULT 'open',           -- 状态: open | closed
    entry_time TEXT NOT NULL DEFAULT '',           -- 入场时间
    exit_time TEXT NOT NULL DEFAULT '',            -- 出场时间
    tags TEXT NOT NULL DEFAULT '[]',               -- 标签 (JSON 数组字符串)
    notes TEXT NOT NULL DEFAULT '',                -- 备注
    images TEXT NOT NULL DEFAULT '[]',             -- 图片列表 (JSON 数组字符串)
    ai_feedback TEXT NOT NULL DEFAULT '',           -- AI 反馈内容
    emotion_before TEXT NOT NULL DEFAULT '',        -- 交易前情绪
    emotion_after TEXT NOT NULL DEFAULT '',         -- 交易后情绪
    confidence INTEGER NOT NULL DEFAULT 5,         -- 信心指数 (1-10)
    created_at TEXT NOT NULL,                      -- 创建时间 (RFC3339)
    updated_at TEXT NOT NULL,                      -- 更新时间 (RFC3339)
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE,
    FOREIGN KEY (plan_id) REFERENCES trade_plan(id) ON DELETE SET NULL
);

-- =============================================
-- 交易总结表：每日/每周交易复盘总结
-- =============================================
CREATE TABLE IF NOT EXISTS trade_summary (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,                    -- 关联账户 ID
    summary_type TEXT NOT NULL DEFAULT 'daily',  -- 总结类型: daily | weekly | monthly
    summary_date TEXT NOT NULL,                  -- 总结日期
    total_trades INTEGER NOT NULL DEFAULT 0,     -- 总交易笔数
    win_trades INTEGER NOT NULL DEFAULT 0,       -- 盈利笔数
    loss_trades INTEGER NOT NULL DEFAULT 0,      -- 亏损笔数
    total_pnl REAL NOT NULL DEFAULT 0.0,         -- 总盈亏
    total_commission REAL NOT NULL DEFAULT 0.0,  -- 总手续费
    net_pnl REAL NOT NULL DEFAULT 0.0,           -- 净盈亏
    win_rate REAL NOT NULL DEFAULT 0.0,           -- 胜率
    avg_profit REAL NOT NULL DEFAULT 0.0,         -- 平均盈利
    avg_loss REAL NOT NULL DEFAULT 0.0,           -- 平均亏损
    profit_factor REAL NOT NULL DEFAULT 0.0,      -- 盈亏比
    max_profit REAL NOT NULL DEFAULT 0.0,         -- 最大单笔盈利
    max_loss REAL NOT NULL DEFAULT 0.0,           -- 最大单笔亏损
    emotion_score INTEGER NOT NULL DEFAULT 5,     -- 情绪评分 (1-10)
    market_view TEXT NOT NULL DEFAULT '',          -- 市场观点
    lessons TEXT NOT NULL DEFAULT '',               -- 经验教训
    improvement TEXT NOT NULL DEFAULT '',            -- 改进计划
    tags TEXT NOT NULL DEFAULT '[]',                -- 标签 (JSON 数组字符串)
    created_at TEXT NOT NULL,                       -- 创建时间 (RFC3339)
    updated_at TEXT NOT NULL,                       -- 更新时间 (RFC3339)
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);

-- =============================================
-- 索引：优化常用查询性能
-- =============================================
CREATE INDEX IF NOT EXISTS idx_account_active ON account(is_active);
CREATE INDEX IF NOT EXISTS idx_trade_plan_account ON trade_plan(account_id);
CREATE INDEX IF NOT EXISTS idx_trade_plan_status ON trade_plan(status);
CREATE INDEX IF NOT EXISTS idx_trade_plan_symbol ON trade_plan(symbol);
CREATE INDEX IF NOT EXISTS idx_trade_log_account ON trade_log(account_id);
CREATE INDEX IF NOT EXISTS idx_trade_log_plan ON trade_log(plan_id);
CREATE INDEX IF NOT EXISTS idx_trade_log_status ON trade_log(status);
CREATE INDEX IF NOT EXISTS idx_trade_log_entry_time ON trade_log(entry_time);
CREATE INDEX IF NOT EXISTS idx_trade_summary_account ON trade_summary(account_id);
CREATE INDEX IF NOT EXISTS idx_trade_summary_date ON trade_summary(summary_date);
CREATE INDEX IF NOT EXISTS idx_trade_summary_type ON trade_summary(summary_type);
