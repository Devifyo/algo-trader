-- ════════════════════════════════════════════════════════════════════════
--  Authentication & roles. SaaS users register (creating a tenant) and log in
--  to their tenant-scoped workspace; platform admins manage everything.
--
--  `users` is intentionally NOT under RLS — login looks a user up by email
--  before any tenant context exists. Tenant isolation for a user's *data* is
--  still enforced by RLS on the data tables once they're logged in.
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS users (
    id                BIGSERIAL PRIMARY KEY,
    tenant_id         UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000001',
    name              TEXT NOT NULL,
    email             TEXT UNIQUE NOT NULL,
    email_verified_at TIMESTAMPTZ,
    password          TEXT NOT NULL,
    -- admin = platform operator (cross-tenant); owner/operator/viewer = tenant roles
    role              TEXT NOT NULL DEFAULT 'owner',
    risk_profile      TEXT NOT NULL DEFAULT 'balanced',
    remember_token    VARCHAR(100),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_users_tenant ON users (tenant_id);

-- The restricted app role reads/writes users (login, profile). Cross-tenant
-- admin reads + tenant creation at signup use the OWNER connection (RLS bypass).
GRANT SELECT, INSERT, UPDATE, DELETE ON users TO algotrader_app;
GRANT USAGE, SELECT ON SEQUENCE users_id_seq TO algotrader_app;

-- A 'plan' on tenants for billing/feature gating (added here to keep 03 stable).
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS owner_email TEXT;
