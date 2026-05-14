-- 交易模板表：保存常用交易计划参数为可复用模板
CREATE TABLE IF NOT EXISTS trade_template (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL DEFAULT '',
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    symbol TEXT NOT NULL DEFAULT '',
    direction TEXT NOT NULL DEFAULT 'long',
    market_type TEXT NOT NULL DEFAULT 'futures',
    strategy TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    stop_loss_ratio REAL NOT NULL DEFAULT 0.0,
    take_profit_ratio REAL NOT NULL DEFAULT 0.0,
    default_lots REAL NOT NULL DEFAULT 0.0,
    notes TEXT NOT NULL DEFAULT '',
    usage_count INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_template_account ON trade_template(account_id);
CREATE INDEX IF NOT EXISTS idx_template_pinned ON trade_template(is_pinned);

-- 性能优化索引
CREATE INDEX IF NOT EXISTS idx_trade_log_symbol_date ON trade_log(symbol, entry_time);
CREATE INDEX IF NOT EXISTS idx_trade_plan_created ON trade_plan(created_at);
CREATE INDEX IF NOT EXISTS idx_trade_log_pnl ON trade_log(pnl);
