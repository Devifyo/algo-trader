-- algo-trader schema. Runs once on first container start.
-- Time-series tables are TimescaleDB hypertables; relational tables are plain.

CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ─────────────────────────── Reference / config ───────────────────────────
CREATE TABLE IF NOT EXISTS exchanges (
    id          SMALLSERIAL PRIMARY KEY,
    name        TEXT UNIQUE NOT NULL,        -- binance, bybit, okx, coinbase
    testnet     BOOLEAN NOT NULL DEFAULT true,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS symbols (
    id            SERIAL PRIMARY KEY,
    exchange_id   SMALLINT NOT NULL REFERENCES exchanges(id),
    symbol        TEXT NOT NULL,             -- BTCUSDT
    base          TEXT NOT NULL,
    quote         TEXT NOT NULL,
    tick_size     NUMERIC NOT NULL,
    step_size     NUMERIC NOT NULL,
    min_notional  NUMERIC NOT NULL DEFAULT 0,
    enabled       BOOLEAN NOT NULL DEFAULT true,
    UNIQUE (exchange_id, symbol)
);

-- Built-in tenant zero (the self-contained paper stack). Matches
-- core::tenant::DEFAULT_TENANT. The full tenants table + RLS arrive in
-- 03_multitenancy.sql.
\set default_tenant '00000000-0000-0000-0000-000000000001'

CREATE TABLE IF NOT EXISTS strategies (
    id          SERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    name        TEXT NOT NULL,              -- trend, momentum, mean_reversion...
    kind        TEXT NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT false,
    params      JSONB NOT NULL DEFAULT '{}', -- per-strategy tunables
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)               -- strategy names unique per tenant
);

-- ─────────────────────────── Time-series (hypertables) ───────────────────────────
CREATE TABLE IF NOT EXISTS candles (
    time        TIMESTAMPTZ NOT NULL,
    exchange    TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    timeframe   TEXT NOT NULL,               -- 1m,5m,15m,1h,4h,1d
    open        DOUBLE PRECISION NOT NULL,
    high        DOUBLE PRECISION NOT NULL,
    low         DOUBLE PRECISION NOT NULL,
    close       DOUBLE PRECISION NOT NULL,
    volume      DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (exchange, symbol, timeframe, time)
);
SELECT create_hypertable('candles', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

CREATE TABLE IF NOT EXISTS ticks (
    time        TIMESTAMPTZ NOT NULL,
    exchange    TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    price       DOUBLE PRECISION NOT NULL,
    qty         DOUBLE PRECISION NOT NULL,
    side        TEXT                         -- buy/sell (taker)
);
SELECT create_hypertable('ticks', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '1 day');

CREATE TABLE IF NOT EXISTS order_books (
    time        TIMESTAMPTZ NOT NULL,
    exchange    TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    bids        JSONB NOT NULL,              -- [[price,qty],...] top N
    asks        JSONB NOT NULL
);
SELECT create_hypertable('order_books', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '1 day');

CREATE TABLE IF NOT EXISTS funding_rates (
    time        TIMESTAMPTZ NOT NULL,
    exchange    TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    rate        DOUBLE PRECISION NOT NULL,
    PRIMARY KEY (exchange, symbol, time)
);
SELECT create_hypertable('funding_rates', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '30 days');

CREATE TABLE IF NOT EXISTS indicators (
    time        TIMESTAMPTZ NOT NULL,
    exchange    TEXT NOT NULL,
    symbol      TEXT NOT NULL,
    timeframe   TEXT NOT NULL,
    name        TEXT NOT NULL,               -- ema_50, rsi_14, atr_14, regime...
    value       DOUBLE PRECISION NOT NULL
);
SELECT create_hypertable('indicators', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

CREATE TABLE IF NOT EXISTS pnl_snapshots (
    time          TIMESTAMPTZ NOT NULL,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    account       TEXT NOT NULL DEFAULT 'default',
    equity_usd    DOUBLE PRECISION NOT NULL,
    realized_usd  DOUBLE PRECISION NOT NULL,
    unrealized_usd DOUBLE PRECISION NOT NULL,
    open_positions INT NOT NULL
);
SELECT create_hypertable('pnl_snapshots', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '30 days');

-- ─────────────────────────── Trading (relational) ───────────────────────────
CREATE TABLE IF NOT EXISTS orders (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    client_id     TEXT UNIQUE NOT NULL,      -- idempotency key
    exchange      TEXT NOT NULL,
    broker        TEXT,
    symbol        TEXT NOT NULL,
    strategy      TEXT,
    side          TEXT NOT NULL,             -- buy/sell
    type          TEXT NOT NULL,             -- market/limit/stop
    qty           DOUBLE PRECISION NOT NULL,
    price         DOUBLE PRECISION,
    stop_price    DOUBLE PRECISION,
    status        TEXT NOT NULL DEFAULT 'new', -- new/submitted/filled/partial/rejected/canceled
    exchange_oid  TEXT,
    reason        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_orders_symbol_time ON orders (exchange, symbol, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);

CREATE TABLE IF NOT EXISTS fills (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    order_id      UUID NOT NULL REFERENCES orders(id),
    exchange      TEXT NOT NULL,
    symbol        TEXT NOT NULL,
    side          TEXT NOT NULL,
    qty           DOUBLE PRECISION NOT NULL,
    price         DOUBLE PRECISION NOT NULL,
    fee           DOUBLE PRECISION NOT NULL DEFAULT 0,
    fee_asset     TEXT,
    filled_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_fills_order ON fills (order_id);

CREATE TABLE IF NOT EXISTS positions (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    exchange      TEXT NOT NULL,
    symbol        TEXT NOT NULL,
    strategy      TEXT,
    side          TEXT NOT NULL,             -- long/short
    qty           DOUBLE PRECISION NOT NULL,
    entry_price   DOUBLE PRECISION NOT NULL,
    stop_price    DOUBLE PRECISION,
    take_profit   DOUBLE PRECISION,
    leverage      DOUBLE PRECISION NOT NULL DEFAULT 1,
    realized_pnl  DOUBLE PRECISION NOT NULL DEFAULT 0,
    status        TEXT NOT NULL DEFAULT 'open', -- open/closed
    opened_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at     TIMESTAMPTZ
);
-- At most one open position per (tenant, exchange, symbol). This partial unique
-- index is also the ON CONFLICT target used by the engine's upsert.
CREATE UNIQUE INDEX IF NOT EXISTS uq_positions_open
    ON positions (tenant_id, exchange, symbol) WHERE status = 'open';

CREATE TABLE IF NOT EXISTS audit_log (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor       TEXT NOT NULL,               -- user email or 'system'
    action      TEXT NOT NULL,               -- kill_switch_on, strategy_enable...
    target      TEXT,
    detail      JSONB
);
CREATE INDEX IF NOT EXISTS idx_audit_at ON audit_log (at DESC);

-- ─────────────────────────── Retention & compression ───────────────────────────
ALTER TABLE ticks       SET (timescaledb.compress, timescaledb.compress_segmentby = 'exchange,symbol');
ALTER TABLE order_books SET (timescaledb.compress, timescaledb.compress_segmentby = 'exchange,symbol');
ALTER TABLE candles     SET (timescaledb.compress, timescaledb.compress_segmentby = 'exchange,symbol,timeframe');
SELECT add_compression_policy('ticks',       INTERVAL '7 days',  if_not_exists => true);
SELECT add_compression_policy('order_books', INTERVAL '2 days',  if_not_exists => true);
SELECT add_compression_policy('candles',     INTERVAL '14 days', if_not_exists => true);
SELECT add_retention_policy('ticks',       INTERVAL '90 days',  if_not_exists => true);
SELECT add_retention_policy('order_books', INTERVAL '30 days',  if_not_exists => true);
