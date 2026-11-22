//! Paper-trading fill simulator. Fills at the latest cached price with a small
//! adverse slippage and a taker fee, so paper PnL approximates reality. This is
//! the default executor until live venue adapters pass the go-live checklist.

use tcore::types::{ExecStatus, Execution, OrderIntent, Side};
use rand::Rng;

const SLIPPAGE_BPS: f64 = 2.0; // adverse
const FEE_BPS: f64 = 5.0; // taker

/// Simulate a fill given the reference price (latest cached or order price).
pub fn simulate(order: &OrderIntent, reference_price: f64) -> Execution {
    let jitter = {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..1.0) // 0..1 bps extra noise
    };
    let slip = (SLIPPAGE_BPS + jitter) / 10_000.0;
    // Buyers pay up, sellers receive less.
    let avg_price = reference_price * (1.0 + slip * order.side.sign());
    let notional = avg_price * order.qty;
    let fee = notional * (FEE_BPS / 10_000.0);

    Execution {
        client_id: order.client_id.clone(),
        tenant: order.tenant,
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        side: order.side,
        status: ExecStatus::Filled,
        filled_qty: order.qty,
        avg_price,
        fee,
        exchange_oid: Some(format!("paper-{}", order.client_id)),
        reason: None,
        ts: chrono::Utc::now(),
    }
}

/// Direction helper for position bookkeeping.
pub fn position_side(order: &OrderIntent) -> Side {
    order.side
}
