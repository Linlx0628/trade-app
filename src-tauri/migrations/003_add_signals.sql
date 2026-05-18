CREATE TABLE IF NOT EXISTS signal_alert (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    symbol TEXT NOT NULL,
    alert_type TEXT NOT NULL,
    condition_value REAL,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    is_triggered INTEGER NOT NULL DEFAULT 0,
    triggered_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_signal_alert_active ON signal_alert(account_id, is_active);
