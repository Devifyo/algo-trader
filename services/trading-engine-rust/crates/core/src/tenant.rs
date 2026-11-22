//! Multi-tenancy primitives. Every signal/order/execution is owned by a tenant
//! and pinned to a region. Tenant isolation is enforced at three layers:
//!   * NATS   — per-tenant subjects (`tenant.{id}.*`) so one tenant's runaway
//!              strategy cannot starve the global execution queues.
//!   * Vault  — per-tenant secret partition (`secret/tenants/{id}/brokers/*`).
//!   * Postgres — Row-Level Security keyed on `tenant_id` (set via
//!              `SET app.tenant_id`). See store::with_tenant.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Tenant zero — the built-in tenant the self-contained paper stack runs as.
/// Fixed UUID so the synthetic flow and seed data line up.
pub const DEFAULT_TENANT: Uuid = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

impl TenantId {
    pub fn default_tenant() -> Self {
        TenantId(DEFAULT_TENANT)
    }
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}
impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Execution region. Spokes are deployed per region to keep broker round-trips
/// under the latency budget and to satisfy data-residency / egress-IP rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    /// AWS us-east-1 — US equities + global crypto.
    UsEast1,
    /// AWS ap-south-1 — India NSE/BSE (SEBI regime).
    ApSouth1,
}

impl Region {
    pub fn from_env() -> Self {
        match std::env::var("REGION").as_deref() {
            Ok("ap-south-1") => Region::ApSouth1,
            _ => Region::UsEast1,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::UsEast1 => "us-east-1",
            Region::ApSouth1 => "ap-south-1",
        }
    }
    /// The compliance regime this region operates under.
    pub fn regime(&self) -> Regime {
        match self {
            Region::UsEast1 => Regime::Finra,
            Region::ApSouth1 => Regime::Sebi,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// US — FinRA Pattern Day Trading constraints below $25k equity.
    Finra,
    /// India — SEBI 2026 algo constraints (order-rate cap, broker whitelist,
    /// static egress IP).
    Sebi,
}

// ── Tenant-scoped naming ────────────────────────────────────────────────────

/// Vault KV-v2 logical path for a tenant's broker credentials.
pub fn vault_broker_path(tenant: TenantId, broker: &str) -> String {
    format!("secret/data/tenants/{tenant}/brokers/{broker}")
}

pub fn subj_signals(tenant: TenantId) -> String {
    format!("tenant.{tenant}.signals")
}
pub fn subj_orders(tenant: TenantId) -> String {
    format!("tenant.{tenant}.orders")
}
pub fn subj_executions(tenant: TenantId) -> String {
    format!("tenant.{tenant}.executions")
}
pub fn subj_positions(tenant: TenantId) -> String {
    format!("tenant.{tenant}.positions")
}
/// Wildcards a regional engine subscribes to across all its tenants.
pub const ALL_TENANT_SIGNALS: &str = "tenant.*.signals";
pub const ALL_TENANT_ORDERS: &str = "tenant.*.orders";
