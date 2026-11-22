-- ════════════════════════════════════════════════════════════════════════
--  News Intelligence & Sentiment. All ADVISORY — feeds sentiment/risk/
--  confidence scores; never triggers a trade (the risk engine is final).
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS news_sources (
    id          SERIAL PRIMARY KEY,
    name        TEXT UNIQUE NOT NULL,
    kind        TEXT NOT NULL,                 -- rss|api|social|onchain
    url         TEXT NOT NULL,
    asset_class TEXT NOT NULL,                 -- crypto|financial|economic|social|onchain
    enabled     BOOLEAN NOT NULL DEFAULT true,
    last_polled TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS news_categories (
    id   SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS news_articles (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source       TEXT NOT NULL,
    url          TEXT NOT NULL,
    url_hash     TEXT UNIQUE NOT NULL,         -- dedupe key
    title        TEXT NOT NULL,
    summary      TEXT,
    published_at TIMESTAMPTZ,
    categories   TEXT[] NOT NULL DEFAULT '{}',
    assets       TEXT[] NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_news_articles_created ON news_articles (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_news_articles_assets ON news_articles USING GIN (assets);

CREATE TABLE IF NOT EXISTS news_ai_analysis (
    id          BIGSERIAL PRIMARY KEY,
    article_id  UUID NOT NULL REFERENCES news_articles(id) ON DELETE CASCADE,
    bias        TEXT NOT NULL,
    score       DOUBLE PRECISION NOT NULL,     -- -100..100
    confidence  DOUBLE PRECISION NOT NULL,     -- 0..100
    label       TEXT,
    providers   JSONB NOT NULL DEFAULT '[]',   -- per-provider verdicts
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS news_sentiments (
    id            BIGSERIAL PRIMARY KEY,
    article_id    UUID NOT NULL REFERENCES news_articles(id) ON DELETE CASCADE,
    sentiment_score DOUBLE PRECISION NOT NULL, -- -100..100
    confidence    DOUBLE PRECISION NOT NULL,   -- 0..100
    market_impact TEXT NOT NULL,               -- low|medium|high
    duration      TEXT,
    risk_level    TEXT,
    assets        TEXT[] NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_news_sentiments_assets ON news_sentiments USING GIN (assets);
CREATE INDEX IF NOT EXISTS idx_news_sentiments_created ON news_sentiments (created_at DESC);

CREATE TABLE IF NOT EXISTS social_mentions (
    time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    asset       TEXT NOT NULL,
    source      TEXT NOT NULL,                 -- x|reddit|telegram
    mentions    BIGINT NOT NULL DEFAULT 0,
    sentiment   DOUBLE PRECISION,              -- -100..100
    trending    TEXT[] DEFAULT '{}'
);
SELECT create_hypertable('social_mentions', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

CREATE TABLE IF NOT EXISTS economic_events (
    id           BIGSERIAL PRIMARY KEY,
    name         TEXT NOT NULL,
    country      TEXT,
    importance   TEXT NOT NULL,                -- low|medium|high
    scheduled_at TIMESTAMPTZ NOT NULL,
    actual       TEXT,
    forecast     TEXT,
    previous     TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (name, scheduled_at)
);
CREATE INDEX IF NOT EXISTS idx_econ_events_sched ON economic_events (scheduled_at);

CREATE TABLE IF NOT EXISTS market_events (
    id          BIGSERIAL PRIMARY KEY,
    article_id  UUID REFERENCES news_articles(id) ON DELETE SET NULL,
    event_type  TEXT NOT NULL,                 -- etf_approval|exchange_hack|...
    severity    TEXT NOT NULL,                 -- warning|critical
    asset       TEXT,
    headline    TEXT,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_market_events_detected ON market_events (detected_at DESC);

CREATE TABLE IF NOT EXISTS onchain_metrics (
    time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    asset       TEXT NOT NULL,
    metric      TEXT NOT NULL,                 -- whale_tx|exchange_inflow|funding_rate|oi|liquidations
    value       DOUBLE PRECISION NOT NULL
);
SELECT create_hypertable('onchain_metrics', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

CREATE TABLE IF NOT EXISTS sentiment_history (
    time            TIMESTAMPTZ NOT NULL DEFAULT now(),
    asset           TEXT NOT NULL,
    sentiment_score DOUBLE PRECISION NOT NULL,
    confidence      DOUBLE PRECISION NOT NULL,
    news_count      INT NOT NULL DEFAULT 0
);
SELECT create_hypertable('sentiment_history', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');
CREATE INDEX IF NOT EXISTS idx_sentiment_history_asset ON sentiment_history (asset, time DESC);

-- Seed categories + sources (matches news-service/app/sources.py).
INSERT INTO news_categories (name) VALUES
 ('Bitcoin'),('Ethereum'),('Solana'),('Altcoins'),('ETF News'),('SEC News'),
 ('Regulations'),('Macro Economics'),('Federal Reserve'),('Interest Rates'),
 ('Inflation'),('Exchange News'),('Security Breaches'),('Stablecoins'),
 ('AI & Crypto'),('Institutional Adoption')
ON CONFLICT (name) DO NOTHING;
