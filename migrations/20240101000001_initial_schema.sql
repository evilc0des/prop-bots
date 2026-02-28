-- Initial schema for prop-bots trading system

-- Instrument definitions
CREATE TABLE IF NOT EXISTS instruments (
    symbol          TEXT PRIMARY KEY,
    asset_class     TEXT NOT NULL CHECK (asset_class IN ('futures', 'cfd', 'crypto')),
    tick_size       DECIMAL NOT NULL,
    tick_value       DECIMAL NOT NULL,
    contract_size   DECIMAL NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'USD',
    exchange        TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Historical bar (OHLCV) data
CREATE TABLE IF NOT EXISTS bars (
    instrument      TEXT NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL,
    open            DECIMAL NOT NULL,
    high            DECIMAL NOT NULL,
    low             DECIMAL NOT NULL,
    close           DECIMAL NOT NULL,
    volume          DECIMAL NOT NULL DEFAULT 0,
    PRIMARY KEY (instrument, timestamp)
);

CREATE INDEX IF NOT EXISTS idx_bars_instrument_ts ON bars (instrument, timestamp);

-- Historical tick data
CREATE TABLE IF NOT EXISTS ticks (
    id              BIGSERIAL PRIMARY KEY,
    instrument      TEXT NOT NULL,
    timestamp       TIMESTAMPTZ NOT NULL,
    bid             DECIMAL NOT NULL,
    ask             DECIMAL NOT NULL,
    last_price      DECIMAL NOT NULL,
    volume          DECIMAL NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_ticks_instrument_ts ON ticks (instrument, timestamp);

-- Trade log
CREATE TABLE IF NOT EXISTS trades (
    id              UUID PRIMARY KEY,
    instrument      TEXT NOT NULL,
    side            TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    quantity        DECIMAL NOT NULL,
    entry_price     DECIMAL NOT NULL,
    exit_price      DECIMAL NOT NULL,
    pnl             DECIMAL NOT NULL,
    commission      DECIMAL NOT NULL DEFAULT 0,
    entry_time      TIMESTAMPTZ NOT NULL,
    exit_time       TIMESTAMPTZ NOT NULL,
    strategy_id     TEXT,
    backtest_id     UUID
);

CREATE INDEX IF NOT EXISTS idx_trades_strategy ON trades (strategy_id);
CREATE INDEX IF NOT EXISTS idx_trades_instrument ON trades (instrument);

-- Order log
CREATE TABLE IF NOT EXISTS orders (
    id              UUID PRIMARY KEY,
    instrument      TEXT NOT NULL,
    side            TEXT NOT NULL,
    order_type      TEXT NOT NULL,
    quantity        DECIMAL NOT NULL,
    filled_quantity DECIMAL NOT NULL DEFAULT 0,
    price           DECIMAL,
    stop_price      DECIMAL,
    status          TEXT NOT NULL,
    strategy_id     TEXT,
    broker_order_id TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Backtest results
CREATE TABLE IF NOT EXISTS backtest_results (
    id                  UUID PRIMARY KEY,
    strategy_id         TEXT NOT NULL,
    instrument          TEXT NOT NULL,
    start_date          TIMESTAMPTZ NOT NULL,
    end_date            TIMESTAMPTZ NOT NULL,
    initial_balance     DECIMAL NOT NULL,
    final_balance       DECIMAL NOT NULL,
    total_trades        BIGINT NOT NULL,
    winning_trades      BIGINT NOT NULL,
    losing_trades       BIGINT NOT NULL,
    gross_profit        DECIMAL NOT NULL,
    gross_loss          DECIMAL NOT NULL,
    net_profit          DECIMAL NOT NULL,
    max_drawdown        DECIMAL NOT NULL,
    max_drawdown_percent DECIMAL NOT NULL,
    win_rate            DECIMAL NOT NULL,
    profit_factor       DECIMAL NOT NULL,
    sharpe_ratio        DECIMAL NOT NULL,
    sortino_ratio       DECIMAL NOT NULL,
    avg_trade_pnl       DECIMAL NOT NULL,
    avg_winner          DECIMAL NOT NULL,
    avg_loser           DECIMAL NOT NULL,
    total_commission    DECIMAL NOT NULL,
    equity_curve        JSONB,
    trades              JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Prop firm rule profiles
CREATE TABLE IF NOT EXISTS prop_firm_profiles (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                    TEXT NOT NULL UNIQUE,
    description             TEXT,
    daily_loss_limit        DECIMAL NOT NULL,
    max_drawdown            DECIMAL NOT NULL,
    trailing_drawdown       BOOLEAN NOT NULL DEFAULT FALSE,
    max_position_size       DECIMAL,
    max_contracts           INTEGER,
    trading_start_utc       TIME,
    trading_end_utc         TIME,
    consistency_rule        BOOLEAN NOT NULL DEFAULT FALSE,
    consistency_max_pct     DECIMAL,
    initial_balance         DECIMAL NOT NULL,
    rules_json              JSONB,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Strategy configurations
CREATE TABLE IF NOT EXISTS strategy_configs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    strategy_type   TEXT NOT NULL,
    parameters      JSONB NOT NULL DEFAULT '{}',
    instrument      TEXT,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
