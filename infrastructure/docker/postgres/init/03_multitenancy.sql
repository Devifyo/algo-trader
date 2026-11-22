-- ════════════════════════════════════════════════════════════════════════
--  Multi-tenancy: tenants, regional config, Row-Level Security.
--
--  Trust model:
--   * The Rust engine connects as the table OWNER (algotrader) and writes
--     tenant_id explicitly. Owners are not subject to RLS here (we do NOT
--     FORCE RLS on the owner) — the engine is trusted data-plane code.
--   * The control plane (Laravel hub) connects as the RESTRICTED login role
--     `algotrader_app`, for which every tenant table is filtered by RLS on
--     `current_setting('app.tenant_id')`. The hub runs
--     `SET app.tenant_id = '<uuid>'` per request (see Laravel TenantScope).
--  This guarantees a tenant can never read or write another tenant's rows
--  through the application layer, even with a SQL-injection bug.
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS tenants (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name          TEXT NOT NULL,
    region        TEXT NOT NULL DEFAULT 'us-east-1',  -- us-east-1 | ap-south-1
    regime        TEXT NOT NULL DEFAULT 'finra',      -- finra | sebi
    plan          TEXT NOT NULL DEFAULT 'starter',
    status        TEXT NOT NULL DEFAULT 'active',     -- active | suspended | closed
    -- SEBI: per-tenant order-rate cap (orders/sec). FinRA tenants use a higher cap.
    max_orders_per_sec INT NOT NULL DEFAULT 10,
    -- Static egress Elastic IP the spoke must use for this tenant's broker calls.
    egress_ip     INET,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Broker linkage: we store only a *reference*; the secret lives in Vault at
-- secret/tenants/{tenant_id}/brokers/{broker}. Never the key itself.
CREATE TABLE IF NOT EXISTS tenant_brokers (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id     UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    broker        TEXT NOT NULL,             -- binance|bybit|zerodha|upstox...
    vault_path    TEXT NOT NULL,             -- secret/tenants/{id}/brokers/{broker}
    enabled       BOOLEAN NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, broker)
);

-- Per-tenant API usage / billing metering.
CREATE TABLE IF NOT EXISTS api_usage (
    time        TIMESTAMPTZ NOT NULL DEFAULT now(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    metric      TEXT NOT NULL,               -- orders, signals, api_calls
    value       BIGINT NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_api_usage_tenant_time ON api_usage (tenant_id, time DESC);

-- Tenant zero (matches core::tenant::DEFAULT_TENANT).
INSERT INTO tenants (id, name, region, regime, plan, max_orders_per_sec)
VALUES ('00000000-0000-0000-0000-000000000001', 'default', 'us-east-1', 'finra', 'internal', 50)
ON CONFLICT (id) DO NOTHING;

-- ── Restricted application role used by the Laravel hub ──
DO $$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'algotrader_app') THEN
      CREATE ROLE algotrader_app LOGIN PASSWORD 'change_me_app_role';
   END IF;
END
$$;
GRANT USAGE ON SCHEMA public TO algotrader_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO algotrader_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO algotrader_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO algotrader_app;

-- ── RLS helper: read the request-scoped tenant id (NULL if unset) ──
CREATE OR REPLACE FUNCTION current_tenant_id() RETURNS UUID AS $$
    SELECT NULLIF(current_setting('app.tenant_id', true), '')::uuid;
$$ LANGUAGE sql STABLE;

-- ── Enable + define RLS on every tenant-scoped table ──
DO $$
DECLARE t TEXT;
BEGIN
  FOREACH t IN ARRAY ARRAY[
      'strategies','orders','fills','positions','pnl_snapshots',
      'audit_log','tenant_brokers','api_usage'
  ] LOOP
      EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY;', t);
      -- Only the restricted app role is forced through RLS; the owner (engine)
      -- bypasses it and writes tenant_id explicitly.
      EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON %I;', t);
      EXECUTE format(
        'CREATE POLICY tenant_isolation ON %I
           USING (tenant_id = current_tenant_id())
           WITH CHECK (tenant_id = current_tenant_id());', t);
  END LOOP;
END
$$;

-- `tenants` itself: an app session may only see its own tenant row.
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_self ON tenants;
CREATE POLICY tenant_self ON tenants
    USING (id = current_tenant_id())
    WITH CHECK (id = current_tenant_id());
