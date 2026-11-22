-- ════════════════════════════════════════════════════════════════════════
--  AI Decision / Explainability log. Every signal the risk engine evaluates is
--  recorded here with its full reasoning — why a trade was opened or skipped:
--  the rule strength, the AI confidence (heuristic now, multi-provider consensus
--  when keys are configured), news sentiment, market regime, and the deciding
--  gate. Powers the "AI Decisions" feed. Per-tenant (RLS like orders).
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS trade_decisions (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       UUID NOT NULL,
    ts              TIMESTAMPTZ NOT NULL DEFAULT now(),
    symbol          TEXT NOT NULL,
    strategy        TEXT NOT NULL,
    side            TEXT NOT NULL,                 -- buy | sell
    signal_strength DOUBLE PRECISION NOT NULL DEFAULT 0,
    ai_confidence   DOUBLE PRECISION,              -- 0..1
    ai_mode         TEXT,                          -- heuristic | consensus
    sentiment       DOUBLE PRECISION,              -- -100..100 (news)
    regime          TEXT,                          -- market regime at decision time
    action          TEXT NOT NULL,                 -- traded | skipped
    reason          TEXT,                          -- deciding gate / rationale
    providers       JSONB                          -- per-provider votes (consensus)
);
CREATE INDEX IF NOT EXISTS idx_trade_decisions_tenant_ts ON trade_decisions (tenant_id, ts DESC);

GRANT SELECT, INSERT ON trade_decisions TO algotrader_app;
GRANT USAGE, SELECT ON SEQUENCE trade_decisions_id_seq TO algotrader_app;

ALTER TABLE trade_decisions ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON trade_decisions;
CREATE POLICY tenant_isolation ON trade_decisions
    USING (tenant_id = current_tenant_id())
    WITH CHECK (tenant_id = current_tenant_id());
