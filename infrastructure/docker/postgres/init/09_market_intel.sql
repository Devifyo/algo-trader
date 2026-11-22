-- ════════════════════════════════════════════════════════════════════════
--  Market Intelligence: the Opportunity Scanner's live output. Market-wide
--  (not per-tenant) — these are shared signals every tenant's portfolio reads.
--  Written by the Rust `scanner` service every few seconds from live candles
--  + news sentiment. Powers the Markets dashboard, rankings and allocation.
-- ════════════════════════════════════════════════════════════════════════

-- Per-market opportunity scores (0..100) + the factor breakdown (explainability).
CREATE TABLE IF NOT EXISTS market_scores (
    symbol           TEXT PRIMARY KEY,
    market_score     DOUBLE PRECISION NOT NULL DEFAULT 0,   -- overall quality 0..100
    bullish_score    DOUBLE PRECISION NOT NULL DEFAULT 0,
    bearish_score    DOUBLE PRECISION NOT NULL DEFAULT 0,
    confidence_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    risk_score       DOUBLE PRECISION NOT NULL DEFAULT 0,    -- higher = riskier
    trend_strength   DOUBLE PRECISION NOT NULL DEFAULT 0,    -- -100..100 (ema slope)
    momentum         DOUBLE PRECISION NOT NULL DEFAULT 0,    -- roc %
    volatility       DOUBLE PRECISION NOT NULL DEFAULT 0,    -- atr % of price
    sentiment        DOUBLE PRECISION NOT NULL DEFAULT 0,    -- -1..1 from news
    price            DOUBLE PRECISION NOT NULL DEFAULT 0,
    bias             TEXT NOT NULL DEFAULT 'neutral',        -- bullish|bearish|neutral
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Single-row market regime snapshot (id = 1).
CREATE TABLE IF NOT EXISTS market_regime (
    id               INT PRIMARY KEY DEFAULT 1,
    regime           TEXT NOT NULL DEFAULT 'sideways_market',   -- bull|bear|sideways
    volatility       TEXT NOT NULL DEFAULT 'low_volatility',    -- high|low_volatility
    breadth          DOUBLE PRECISION NOT NULL DEFAULT 0,       -- % of universe bullish
    avg_volatility   DOUBLE PRECISION NOT NULL DEFAULT 0,
    favored          TEXT[] NOT NULL DEFAULT '{}',              -- strategies suited to regime
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT market_regime_singleton CHECK (id = 1)
);

-- Dynamic capital allocation toward the best opportunities (+ cash reserve).
CREATE TABLE IF NOT EXISTS capital_allocation (
    symbol           TEXT PRIMARY KEY,                          -- or 'cash_reserve'
    weight           DOUBLE PRECISION NOT NULL DEFAULT 0,       -- 0..1
    rank             INT NOT NULL DEFAULT 0,
    reason           TEXT,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

GRANT SELECT ON market_scores, market_regime, capital_allocation TO algotrader_app;
GRANT INSERT, UPDATE, DELETE ON market_scores, market_regime, capital_allocation TO algotrader_app;
