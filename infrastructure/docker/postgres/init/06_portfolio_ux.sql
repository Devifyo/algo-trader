-- ════════════════════════════════════════════════════════════════════════
--  Advanced AI Portfolio Management + Marketplace + Copy Trading +
--  Safety/UX (feature flags, help center, onboarding, exchange-key safety).
--  Private per-user tables get RLS; marketplace listings + trader profiles are
--  intentionally world-readable (browseable) but carry an owner tenant_id.
-- ════════════════════════════════════════════════════════════════════════

-- ── Bots & portfolios ────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS bots (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id    UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    name         TEXT NOT NULL,
    type         TEXT NOT NULL,             -- trend_following|scalping|mean_reversion|...
    strategy     TEXT,
    exchange     TEXT,
    status       TEXT NOT NULL DEFAULT 'paper',   -- paper|forward|live|paused
    config       JSONB NOT NULL DEFAULT '{}',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS portfolios (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id    UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    name         TEXT NOT NULL,
    risk_profile TEXT NOT NULL DEFAULT 'balanced',
    equity_usd   DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS risk_profiles (
    id          SERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    name        TEXT NOT NULL,
    params      JSONB NOT NULL DEFAULT '{}',
    UNIQUE (tenant_id, name)
);

CREATE TABLE IF NOT EXISTS capital_allocations (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    target_kind TEXT NOT NULL,             -- strategy|bot|asset|exchange
    target_id   TEXT NOT NULL,
    weight      DOUBLE PRECISION NOT NULL,
    amount_usd  DOUBLE PRECISION NOT NULL,
    rebalance   TEXT NOT NULL DEFAULT 'weekly',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Explainable AI decisions (mandatory: never a black box) ──────────────
CREATE TABLE IF NOT EXISTS ai_decisions (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    bot_id        UUID,
    symbol        TEXT,
    decision      TEXT NOT NULL,           -- allow|hold|reject
    bullish_score DOUBLE PRECISION,
    bearish_score DOUBLE PRECISION,
    risk_score    DOUBLE PRECISION,
    confidence    DOUBLE PRECISION,
    market_score  DOUBLE PRECISION,
    portfolio_score DOUBLE PRECISION,
    explanation   JSONB NOT NULL DEFAULT '{}',   -- reason/data_used/risk_factors/expected_outcome
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_ai_decisions_tenant_time ON ai_decisions (tenant_id, created_at DESC);

CREATE TABLE IF NOT EXISTS strategy_health (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    strategy      TEXT NOT NULL,
    health_score  DOUBLE PRECISION,
    risk_score    DOUBLE PRECISION,
    flags         TEXT[] DEFAULT '{}',
    metrics       JSONB NOT NULL DEFAULT '{}',
    evaluated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS trade_recovery (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    position_id   UUID,
    action        TEXT NOT NULL,
    explanation   JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS optimization_runs (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    strategy      TEXT NOT NULL,
    method        TEXT NOT NULL,
    optimized     JSONB NOT NULL DEFAULT '{}',
    expected_return DOUBLE PRECISION,
    confidence    DOUBLE PRECISION,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Marketplace (browseable listings) ────────────────────────────────────
CREATE TABLE IF NOT EXISTS marketplace_strategies (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    seller_tenant_id UUID NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    pricing_model   TEXT NOT NULL DEFAULT 'subscription',  -- subscription|rent|sale|free
    price_usd       NUMERIC NOT NULL DEFAULT 0,
    monthly_return  DOUBLE PRECISION,
    annual_return   DOUBLE PRECISION,
    max_drawdown    DOUBLE PRECISION,
    win_rate        DOUBLE PRECISION,
    risk_score      DOUBLE PRECISION,
    subscribers     INT NOT NULL DEFAULT 0,
    rating          DOUBLE PRECISION NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'published',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS marketplace_subscriptions (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    strategy_id   UUID NOT NULL REFERENCES marketplace_strategies(id) ON DELETE CASCADE,
    status        TEXT NOT NULL DEFAULT 'active',
    started_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS strategy_reviews (
    id            BIGSERIAL PRIMARY KEY,
    strategy_id   UUID NOT NULL REFERENCES marketplace_strategies(id) ON DELETE CASCADE,
    reviewer_tenant_id UUID NOT NULL,
    rating        INT NOT NULL CHECK (rating BETWEEN 1 AND 5),
    review        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Copy trading ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS copy_traders (
    id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id      UUID NOT NULL,            -- the trader being copied
    display_name   TEXT NOT NULL,
    verified       BOOLEAN NOT NULL DEFAULT false,
    risk_rating    TEXT,                     -- low|medium|high
    followers      INT NOT NULL DEFAULT 0,
    monthly_return DOUBLE PRECISION,
    max_drawdown   DOUBLE PRECISION,
    win_rate       DOUBLE PRECISION,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS copy_followers (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',  -- follower
    trader_id     UUID NOT NULL REFERENCES copy_traders(id) ON DELETE CASCADE,
    copy_mode     TEXT NOT NULL DEFAULT 'percentage_based',  -- fixed_amount|percentage_based|risk_multiplier
    amount_usd    DOUBLE PRECISION,
    multiplier    DOUBLE PRECISION DEFAULT 1.0,
    max_drawdown_stop DOUBLE PRECISION DEFAULT 20.0,
    status        TEXT NOT NULL DEFAULT 'active',
    started_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS copy_trades (
    id            BIGSERIAL PRIMARY KEY,
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',  -- follower
    follower_id   UUID NOT NULL REFERENCES copy_followers(id) ON DELETE CASCADE,
    symbol        TEXT NOT NULL,
    side          TEXT NOT NULL,
    qty           DOUBLE PRECISION NOT NULL,
    status        TEXT NOT NULL DEFAULT 'mirrored',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ── Exchange-key safety (NEVER stores the secret — Vault ref only) ───────
CREATE TABLE IF NOT EXISTS exchange_keys (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    exchange      TEXT NOT NULL,
    label         TEXT NOT NULL,
    vault_path    TEXT NOT NULL,            -- secret/tenants/{id}/brokers/{exchange}
    permissions   JSONB NOT NULL DEFAULT '{}',  -- safety breakdown
    withdraw_enabled BOOLEAN NOT NULL DEFAULT false,
    ip_whitelisted BOOLEAN NOT NULL DEFAULT false,
    safe          BOOLEAN NOT NULL DEFAULT false,
    last_checked  TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Hard safety constraint at the database layer: refuse to persist a
    -- withdrawal-enabled key (mirrors the safety layer + docs/SECURITY.md).
    CONSTRAINT no_withdraw_keys CHECK (withdraw_enabled = false)
);

-- ── Feature flags (global config; scopes resolved in app) ────────────────
CREATE TABLE IF NOT EXISTS feature_flags (
    id          SERIAL PRIMARY KEY,
    flag        TEXT NOT NULL,
    scope       TEXT NOT NULL DEFAULT 'global',  -- global|plan|tenant|user
    scope_id    TEXT NOT NULL DEFAULT '*',
    enabled     BOOLEAN NOT NULL DEFAULT true,
    description TEXT,
    UNIQUE (flag, scope, scope_id)
);

-- ── Help center + onboarding (UX) ────────────────────────────────────────
CREATE TABLE IF NOT EXISTS help_articles (
    id          SERIAL PRIMARY KEY,
    slug        TEXT UNIQUE NOT NULL,
    title       TEXT NOT NULL,
    category    TEXT NOT NULL,
    audience    TEXT NOT NULL DEFAULT 'beginner',  -- beginner|advanced
    body        TEXT NOT NULL,
    related     TEXT[] DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS onboarding_progress (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    user_email  TEXT,
    step        TEXT NOT NULL,            -- create_account|choose_risk_profile|connect_exchange|...
    completed   BOOLEAN NOT NULL DEFAULT false,
    completed_at TIMESTAMPTZ,
    UNIQUE (tenant_id, user_email, step)
);

-- ── RLS on private tables ────────────────────────────────────────────────
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY[
      'bots','portfolios','risk_profiles','capital_allocations','ai_decisions',
      'strategy_health','trade_recovery','optimization_runs','marketplace_subscriptions',
      'copy_followers','copy_trades','exchange_keys','onboarding_progress'
  ] LOOP
      EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY;', t);
      EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON %I;', t);
      EXECUTE format(
        'CREATE POLICY tenant_isolation ON %I
           USING (tenant_id = current_tenant_id())
           WITH CHECK (tenant_id = current_tenant_id());', t);
  END LOOP;
END
$$;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO algotrader_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO algotrader_app;

-- ── Seeds ────────────────────────────────────────────────────────────────
INSERT INTO feature_flags (flag, enabled, description) VALUES
 ('copy_trading', true, 'Copy successful traders'),
 ('marketplace', true, 'Buy/sell/rent strategies'),
 ('strategy_optimizer', true, 'Auto-optimize strategy parameters'),
 ('ai_consensus', true, 'Multi-AI consensus analysis'),
 ('paper_trading', true, 'Risk-free simulated trading'),
 ('multi_bot_management', true, 'Run multiple bots as one portfolio')
ON CONFLICT (flag, scope, scope_id) DO NOTHING;

INSERT INTO risk_profiles (name, params) VALUES
 ('conservative', '{"risk_per_trade_pct":0.25,"max_leverage":2,"diversification":"high"}'),
 ('balanced',     '{"risk_per_trade_pct":0.5,"max_leverage":5,"diversification":"medium"}'),
 ('aggressive',   '{"risk_per_trade_pct":1.0,"max_leverage":10,"diversification":"low"}')
ON CONFLICT (tenant_id, name) DO NOTHING;

INSERT INTO help_articles (slug, title, category, audience, body) VALUES
 ('what-is-paper-trading','What is Paper Trading?','getting-started','beginner',
  'Paper trading runs your bots against real market data with a virtual balance — no real money at risk. Use it to validate a strategy (we recommend 30 days and 100+ trades) before going live.'),
 ('how-copy-trading-works','How does Copy Trading work?','copy-trading','beginner',
  'Copy trading mirrors a verified trader''s positions into your account, sized by your chosen mode (fixed amount, percentage, or risk multiplier). Built-in max-drawdown protection stops copying if losses exceed your limit.'),
 ('how-risk-management-works','How does Risk Management work?','risk','beginner',
  'The AI Risk Manager sizes each trade to risk only a small % of your equity, sets a stop-loss, and reduces or halts trading during high volatility, major news, or outages. The Risk Engine always has the final say.'),
 ('what-is-market-regime','What is Market Regime Detection?','ai','beginner',
  'Market regime detection classifies the market (bull, bear, sideways, volatile…) and favors the strategies that work best in that regime.'),
 ('how-ai-sentiment-works','How does AI Sentiment Analysis work?','ai','beginner',
  'We collect financial and crypto news, ask multiple AI providers to score sentiment, and combine them into a consensus. Sentiment only informs confidence and risk — it never triggers a trade by itself.'),
 ('how-to-connect-binance','How do I connect Binance?','exchanges','beginner',
  'Create an API key on Binance with trading enabled and withdrawals DISABLED, restrict it to our server IP, then paste it here. We verify the permissions and refuse any key that can withdraw funds.'),
 ('what-are-api-permissions','What are API Permissions?','exchanges','beginner',
  'API permissions control what a key can do: read, spot/margin/futures trading, and withdrawals. For safety, enable only trading and keep withdrawals OFF.')
ON CONFLICT (slug) DO NOTHING;
