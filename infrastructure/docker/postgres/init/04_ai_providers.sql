-- ════════════════════════════════════════════════════════════════════════
--  Multi-AI Provider Management. Provider + key config is platform-level
--  (managed in the Laravel admin); secrets live in Vault / env, never here.
--  Usage/failure/cost telemetry is per-tenant for billing + analytics.
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS ai_providers (
    id          SERIAL PRIMARY KEY,
    name        TEXT UNIQUE NOT NULL,         -- anthropic | openai | gemini | ...
    enabled     BOOLEAN NOT NULL DEFAULT true,
    priority    INT NOT NULL DEFAULT 100,      -- lower = preferred in failover
    weight      INT NOT NULL DEFAULT 10,       -- weighted load balancing
    base_url    TEXT,                          -- override (OpenAI-compatible/Ollama)
    timeout_ms  INT NOT NULL DEFAULT 30000,
    -- models: {"_tiers":{"default":"<id>","fast":"<id>"}, "<id>":{"input_per_mtok":x,"output_per_mtok":y}}
    models      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS ai_provider_keys (
    id                  SERIAL PRIMARY KEY,
    provider_id         INT NOT NULL REFERENCES ai_providers(id) ON DELETE CASCADE,
    label               TEXT NOT NULL,
    -- secret reference, NOT the secret: "ENV_VAR_NAME" or "vault:<path>#<field>"
    secret_ref          TEXT NOT NULL,
    enabled             BOOLEAN NOT NULL DEFAULT true,
    daily_limit_usd     NUMERIC,
    monthly_limit_usd   NUMERIC,
    status              TEXT NOT NULL DEFAULT 'healthy',  -- healthy|circuit_open|disabled
    consecutive_failures INT NOT NULL DEFAULT 0,
    circuit_open_until  TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider_id, label)
);

CREATE TABLE IF NOT EXISTS ai_requests (
    time          TIMESTAMPTZ NOT NULL DEFAULT now(),
    tenant_id     UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    key_id        INT,
    kind          TEXT NOT NULL,               -- sentiment | risk | regime ...
    latency_ms    INT NOT NULL DEFAULT 0,
    input_tokens  INT NOT NULL DEFAULT 0,
    output_tokens INT NOT NULL DEFAULT 0,
    cost_usd      NUMERIC NOT NULL DEFAULT 0,
    success       BOOLEAN NOT NULL DEFAULT true,
    error         TEXT
);
SELECT create_hypertable('ai_requests', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');
CREATE INDEX IF NOT EXISTS idx_ai_requests_provider_time ON ai_requests (provider, time DESC);

CREATE TABLE IF NOT EXISTS ai_failures (
    time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    provider    TEXT NOT NULL,
    key_id      INT,
    error       TEXT
);
SELECT create_hypertable('ai_failures', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '7 days');

-- Daily rollup for budgets/dashboards (populated by a scheduled job or view).
CREATE TABLE IF NOT EXISTS ai_usage_logs (
    day         DATE NOT NULL,
    provider    TEXT NOT NULL,
    key_id      INT,
    requests    BIGINT NOT NULL DEFAULT 0,
    input_tokens  BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cost_usd    NUMERIC NOT NULL DEFAULT 0,
    PRIMARY KEY (day, provider, key_id)
);

-- Budget tracking per scope (global/provider/tenant/user) per period.
CREATE TABLE IF NOT EXISTS ai_cost_tracking (
    id          SERIAL PRIMARY KEY,
    scope       TEXT NOT NULL,                 -- global|provider|tenant|user
    scope_id    TEXT NOT NULL DEFAULT '*',
    period      TEXT NOT NULL,                 -- daily|weekly|monthly
    budget_usd  NUMERIC NOT NULL,
    spent_usd   NUMERIC NOT NULL DEFAULT 0,
    window_start DATE NOT NULL DEFAULT current_date,
    UNIQUE (scope, scope_id, period, window_start)
);

CREATE TABLE IF NOT EXISTS ai_consensus_results (
    time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    tenant_id   UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    symbol      TEXT,
    signal      TEXT,                          -- sentiment | trade | ...
    bias        TEXT NOT NULL,
    confidence  DOUBLE PRECISION NOT NULL,
    providers   JSONB NOT NULL DEFAULT '[]'
);
SELECT create_hypertable('ai_consensus_results', 'time', if_not_exists => true, chunk_time_interval => INTERVAL '30 days');

-- ── Seed providers with current model pricing ($/1M tokens). Keys are seeded
--    disabled; add real keys (or Vault refs) in the admin to activate. ──
INSERT INTO ai_providers (name, enabled, priority, weight, models) VALUES
 ('anthropic', true, 1, 20, '{
    "_tiers": {"default":"claude-opus-4-8","balanced":"claude-sonnet-4-6","fast":"claude-haiku-4-5"},
    "claude-opus-4-8":   {"input_per_mtok":5.0,  "output_per_mtok":25.0},
    "claude-sonnet-4-6": {"input_per_mtok":3.0,  "output_per_mtok":15.0},
    "claude-haiku-4-5":  {"input_per_mtok":1.0,  "output_per_mtok":5.0}
  }'::jsonb),
 ('openai', true, 2, 10, '{
    "_tiers": {"default":"gpt-4o-mini","balanced":"gpt-4o"},
    "gpt-4o":      {"input_per_mtok":2.5,  "output_per_mtok":10.0},
    "gpt-4o-mini": {"input_per_mtok":0.15, "output_per_mtok":0.6}
  }'::jsonb),
 ('gemini', true, 3, 70, '{
    "_tiers": {"default":"gemini-2.5-flash","balanced":"gemini-2.5-pro"},
    "gemini-2.5-pro":        {"input_per_mtok":1.25, "output_per_mtok":10.0},
    "gemini-2.5-flash":      {"input_per_mtok":0.30, "output_per_mtok":2.50},
    "gemini-2.0-flash":      {"input_per_mtok":0.10, "output_per_mtok":0.40},
    "gemini-2.5-flash-lite": {"input_per_mtok":0.10, "output_per_mtok":0.40}
  }'::jsonb),
 ('deepseek', true, 4, 15, '{
    "_tiers": {"default":"deepseek-chat","balanced":"deepseek-reasoner"},
    "deepseek-chat":     {"input_per_mtok":0.27, "output_per_mtok":1.1},
    "deepseek-reasoner": {"input_per_mtok":0.55, "output_per_mtok":2.19}
  }'::jsonb)
ON CONFLICT (name) DO NOTHING;

-- No placeholder keys are seeded — admins add real keys in the dashboard, which
-- writes the secret to Vault and stores only a reference here.

-- Router failover tuning (single-row), admin-editable in the dashboard. The
-- AI router reads these live (no restart): how many times to retry one key,
-- how many consecutive failures trip a key's circuit, and how long the circuit
-- stays open before the key is retried automatically.
CREATE TABLE IF NOT EXISTS ai_router_settings (
    id                   INTEGER PRIMARY KEY DEFAULT 1,
    retries_per_key      INTEGER NOT NULL DEFAULT 1,
    max_failures         INTEGER NOT NULL DEFAULT 3,
    circuit_open_seconds INTEGER NOT NULL DEFAULT 60,
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT ai_router_settings_singleton CHECK (id = 1)
);
INSERT INTO ai_router_settings (id) VALUES (1) ON CONFLICT (id) DO NOTHING;

-- Strategy-research run history (shown in the admin Strategies page): each deep-
-- research/AI run that proposes which strategy to trade platform-wide.
CREATE TABLE IF NOT EXISTS strategy_research_runs (
    id           SERIAL PRIMARY KEY,
    source       TEXT NOT NULL DEFAULT 'deep-research',
    status       TEXT NOT NULL DEFAULT 'running',   -- running | completed | failed
    recommended  TEXT,
    summary      TEXT,
    started_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

-- Admin-designated platform default strategy (single row), shown across the app.
CREATE TABLE IF NOT EXISTS platform_strategy (
    id               INTEGER PRIMARY KEY DEFAULT 1,
    default_strategy TEXT,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT platform_strategy_singleton CHECK (id = 1)
);
INSERT INTO platform_strategy (id, default_strategy) VALUES (1, 'trend_following')
ON CONFLICT (id) DO NOTHING;

-- Indicative backtest results (run from the admin Strategies page).
CREATE TABLE IF NOT EXISTS strategy_backtests (
    id         SERIAL PRIMARY KEY,
    strategy   TEXT NOT NULL,
    symbol     TEXT NOT NULL,
    timeframe  TEXT NOT NULL,
    bars       INTEGER,
    metrics    JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Public "Contact us" submissions from the marketing site.
CREATE TABLE IF NOT EXISTS contact_messages (
    id         SERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    email      TEXT NOT NULL,
    subject    TEXT NOT NULL,
    message    TEXT NOT NULL,
    handled    BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
