-- ════════════════════════════════════════════════════════════════════════
--  In-app + email notifications. Written by the engine (e.g. a bot reaching its
--  allocation limit) and surfaced in the dashboard bell. `emailed_at` lets the
--  dashboard send each one exactly once via Laravel Mail. Per-tenant (RLS).
-- ════════════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS notifications (
    id          BIGSERIAL PRIMARY KEY,
    tenant_id   UUID NOT NULL,
    kind        TEXT NOT NULL,                 -- allocation_full | system | ...
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    severity    TEXT NOT NULL DEFAULT 'info',  -- info | warning | critical
    read_at     TIMESTAMPTZ,
    emailed_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_notifications_tenant ON notifications (tenant_id, created_at DESC);

GRANT SELECT, INSERT, UPDATE ON notifications TO algotrader_app;
GRANT USAGE, SELECT ON SEQUENCE notifications_id_seq TO algotrader_app;

ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON notifications;
CREATE POLICY tenant_isolation ON notifications
    USING (tenant_id = current_tenant_id())
    WITH CHECK (tenant_id = current_tenant_id());
