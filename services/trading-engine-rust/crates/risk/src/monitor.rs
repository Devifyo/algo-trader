//! Position monitor — enforces protective STOPS, TAKE-PROFITS, and an ATR-style
//! TRAILING stop. Critical: without this, `stop_price`/`take_profit` are recorded
//! on every position but NEVER fire (positions only exited on an opposite
//! signal), leaving open downside. Every few seconds this ratchets each open
//! position's stop toward price and emits a reduce-only market exit the moment
//! price crosses the (trailing) stop or hits the take-profit.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tcore::tenant::{self, TenantId};
use tcore::types::{OrderIntent, OrderType, Side};
use tcore::{metrics, Context, Result};

const CHECK_SECS: u64 = 5;
const REEMIT_SECS: u64 = 60; // retry an exit if the position is still open after this

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    tracing::info!("position monitor starting (stop / take-profit / ATR trailing)");
    let mut tick = tokio::time::interval(Duration::from_secs(CHECK_SECS));
    let mut emitted: HashMap<uuid::Uuid, Instant> = HashMap::new();

    loop {
        tick.tick().await;
        let positions = match ctx.store.all_open_positions().await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "monitor: could not load open positions");
                continue;
            }
        };
        // Forget positions that have since closed (so a future re-open can re-arm).
        let open_ids: std::collections::HashSet<uuid::Uuid> = positions.iter().map(|p| p.0).collect();
        emitted.retain(|id, _| open_ids.contains(id));

        for (id, tenant_uuid, exchange, symbol, strategy, side_s, qty, entry, stop_opt, tp_opt) in positions {
            // Don't spam exits for a position we already told to close (until REEMIT_SECS).
            if let Some(t) = emitted.get(&id) {
                if t.elapsed() < Duration::from_secs(REEMIT_SECS) {
                    continue;
                }
            }
            let price = match ctx.store.get_price(&exchange, &symbol).await {
                Ok(Some(p)) if p > 0.0 => p,
                _ => continue,
            };
            let is_long = side_s == "long";
            let mut hit = false;

            // Take-profit.
            if let Some(tp) = tp_opt {
                if (is_long && price >= tp) || (!is_long && price <= tp) {
                    hit = true;
                }
            }
            // Trailing stop: trail distance = the original |entry - stop| (the ATR
            // stop set at entry). Ratchet toward price; exit when price crosses it.
            if !hit {
                if let Some(stop) = stop_opt {
                    let dist = (entry - stop).abs();
                    let new_stop = if dist > 0.0 {
                        if is_long { (price - dist).max(stop) } else { (price + dist).min(stop) }
                    } else {
                        stop
                    };
                    if (is_long && price <= new_stop) || (!is_long && price >= new_stop) {
                        hit = true;
                    } else if (is_long && new_stop > stop) || (!is_long && new_stop < stop) {
                        let _ = ctx.store.update_position_stop(id, new_stop).await; // ratchet only
                    }
                }
            }
            if !hit {
                continue;
            }

            // Emit a reduce-only market exit (opposite side, full size). Execution
            // re-resolves the real broker; reduce-only is exempt from min-notional
            // and buying-power gates, so a protective exit always gets through.
            let tenant = TenantId(tenant_uuid);
            let order = OrderIntent {
                client_id: format!("stop-{id}"),
                tenant,
                strategy,
                broker: exchange.clone(),
                exchange,
                symbol: symbol.clone(),
                side: if is_long { Side::Sell } else { Side::Buy },
                order_type: OrderType::Market,
                qty,
                price: Some(price),
                stop_price: None,
                take_profit: None,
                leverage: 1.0,
                reduce_only: true,
                ts: chrono::Utc::now(),
            };
            match ctx.bus.publish(tenant::subj_orders(tenant), &order).await {
                Ok(_) => {
                    emitted.insert(id, Instant::now());
                    metrics::inc("protective_exits_total");
                    tracing::info!(%symbol, price, "monitor: protective stop/TP exit emitted");
                }
                Err(e) => tracing::warn!(error = %e, %symbol, "monitor: failed to publish exit"),
            }
        }
    }
}
