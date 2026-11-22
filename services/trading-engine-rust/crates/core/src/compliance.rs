//! Regional regulatory guards enforced inside the risk service, before any
//! order is approved. These are *hard* gates: a violation rejects the order.
//!
//!   * FinRA PDT (US): accounts under $25k may not place a 4th day-trade within
//!     any rolling 5 business days.
//!   * SEBI 2026 (India): per-tenant order-rate cap (default 10/s) to stay below
//!     the HFT classification threshold, plus an approved-broker whitelist.
//!     Static egress Elastic IP whitelisting is enforced at the infra layer
//!     (see infrastructure/kubernetes + docs/COMPLIANCE.md).

use crate::tenant::{Regime, TenantId};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Denial {
    PatternDayTrading,
    OrderRateExceeded,
    BrokerNotWhitelisted,
}

impl Denial {
    pub fn reason(&self) -> &'static str {
        match self {
            Denial::PatternDayTrading => "FINRA_PDT_BLOCK",
            Denial::OrderRateExceeded => "SEBI_ORDER_RATE_EXCEEDED",
            Denial::BrokerNotWhitelisted => "BROKER_NOT_WHITELISTED",
        }
    }
}

/// Per-tenant sliding-window order-rate limiter. SEBI defaults to 10 orders/s.
pub struct OrderRateLimiter {
    max_per_sec: usize,
    window: Duration,
    hits: HashMap<TenantId, VecDeque<Instant>>,
}

impl OrderRateLimiter {
    pub fn new(max_per_sec: usize) -> Self {
        Self {
            max_per_sec,
            window: Duration::from_secs(1),
            hits: HashMap::new(),
        }
    }

    /// Record an attempt; returns `true` if it is within the rate cap.
    pub fn allow(&mut self, tenant: TenantId, now: Instant) -> bool {
        let q = self.hits.entry(tenant).or_default();
        while let Some(&front) = q.front() {
            if now.duration_since(front) > self.window {
                q.pop_front();
            } else {
                break;
            }
        }
        if q.len() >= self.max_per_sec {
            return false;
        }
        q.push_back(now);
        true
    }
}

/// FinRA Pattern-Day-Trading guard. Tracks day-trade timestamps per tenant.
pub struct PdtGuard {
    threshold_equity: f64,
    max_day_trades: usize,
    window: Duration, // ~5 business days
    trades: HashMap<TenantId, VecDeque<Instant>>,
}

impl Default for PdtGuard {
    fn default() -> Self {
        Self {
            threshold_equity: 25_000.0,
            max_day_trades: 3, // a 4th within the window is blocked
            window: Duration::from_secs(5 * 24 * 60 * 60),
            trades: HashMap::new(),
        }
    }
}

impl PdtGuard {
    /// Returns `Ok(())` if a new day-trade is permitted. Above the equity
    /// threshold PDT does not apply.
    pub fn check(&mut self, tenant: TenantId, equity: f64, now: Instant) -> Result<(), Denial> {
        if equity >= self.threshold_equity {
            return Ok(());
        }
        let q = self.trades.entry(tenant).or_default();
        while let Some(&front) = q.front() {
            if now.duration_since(front) > self.window {
                q.pop_front();
            } else {
                break;
            }
        }
        if q.len() >= self.max_day_trades {
            Err(Denial::PatternDayTrading)
        } else {
            Ok(())
        }
    }

    /// Record that a day-trade actually opened (call after approval).
    pub fn record(&mut self, tenant: TenantId, now: Instant) {
        self.trades.entry(tenant).or_default().push_back(now);
    }
}

/// SEBI approved/empanelled broker whitelist. Only these may receive orders in
/// the India region.
pub fn broker_whitelisted(regime: Regime, broker: &str) -> Result<(), Denial> {
    if regime != Regime::Sebi {
        return Ok(());
    }
    // Empanelled broker API identifiers (match the brokers catalog, India market).
    const APPROVED: &[&str] = &[
        "dhan", "zerodha", "upstox", "angelone", "fyers", "shoonya", "flattrade", "fivepaisa",
    ];
    if APPROVED.contains(&broker) {
        Ok(())
    } else {
        Err(Denial::BrokerNotWhitelisted)
    }
}
