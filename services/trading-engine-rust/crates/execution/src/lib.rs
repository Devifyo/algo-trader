//! Execution service. Consumes risk-approved orders on `tenant.*.orders`,
//! routes them (paper simulator by default; live venue adapters when enabled),
//! persists order + fill + position with the owning `tenant_id`, and broadcasts
//! `executions.fill` / `positions.update`.

mod exchanges;
mod paper;

use tcore::bus::decode;
use tcore::config::TradingMode;
use tcore::tenant::{self};
use tcore::types::{ExecStatus, Execution, OrderIntent, PositionUpdate, Side};
use tcore::{metrics, subjects, vault, Context, Result};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    tracing::info!(mode = ?ctx.config.trading_mode, "execution service starting");
    let mut sub = ctx.bus.subscribe(tenant::ALL_TENANT_ORDERS).await?;
    while let Some(msg) = sub.next().await {
        let order: OrderIntent = match decode(&msg.payload) {
            Some(o) => o,
            None => continue,
        };
        if let Err(e) = handle(&ctx, &order).await {
            tracing::error!(error = %e, client_id = %order.client_id, "execution failed");
        }
    }
    Ok(())
}

async fn handle(ctx: &Context, order: &OrderIntent) -> Result<()> {
    let started = Instant::now();
    metrics::inc("orders_submitted_total");

    // Defense-in-depth: never fill while the global kill-switch or this tenant's
    // halt is engaged, even if an order was already queued on the bus.
    let halted = ctx.store.kill_switch().await.unwrap_or(true)
        || ctx.store.tenant_halted(order.tenant).await.unwrap_or(true);
    if halted {
        let canceled = Execution {
            client_id: order.client_id.clone(),
            tenant: order.tenant,
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            side: order.side,
            status: ExecStatus::Canceled,
            filled_qty: 0.0,
            avg_price: 0.0,
            fee: 0.0,
            exchange_oid: None,
            reason: Some("KILL_SWITCH_OR_TENANT_HALT".to_string()),
            ts: chrono::Utc::now(),
        };
        let order_id = ctx.store.insert_order(order).await?;
        ctx.store.record_execution(order_id, &canceled).await?;
        metrics::inc_labeled("orders_rejected_total", "reason", "kill_switch".into());
        ctx.bus.publish(subjects::EXEC_REJECT, &canceled).await?;
        return Ok(());
    }

    // Reference price: latest cached, else the order's own price.
    let reference = ctx
        .store
        .get_price(&order.exchange, &order.symbol)
        .await
        .ok()
        .flatten()
        .or(order.price)
        .unwrap_or(0.0);
    if reference <= 0.0 {
        tracing::warn!(symbol = %order.symbol, "no reference price; skipping");
        return Ok(());
    }

    // Resolve the bot's broker once and stamp it on the order so logs show where
    // it actually routed (paper sim vs a real broker) — not the placeholder.
    let broker = match ctx.config.trading_mode {
        TradingMode::Paper => "paper".to_string(),
        // Live: route to the bot's CONFIGURED venue. A paper-configured bot returns
        // "paper" (simulated, intended). But if no venue can be resolved, NEVER
        // silently fall back to the paper simulator — that mixes fake fills into a
        // real account. Reject cleanly so real and paper stay strictly separate.
        TradingMode::Live => match ctx
            .store
            .bot_broker_for_strategy(order.tenant, &order.strategy)
            .await
            .ok()
            .flatten()
        {
            Some(b) => b,
            None => {
                let rej = Execution {
                    client_id: order.client_id.clone(),
                    tenant: order.tenant,
                    exchange: order.exchange.clone(),
                    symbol: order.symbol.clone(),
                    side: order.side,
                    status: ExecStatus::Rejected,
                    filled_qty: 0.0,
                    avg_price: 0.0,
                    fee: 0.0,
                    exchange_oid: None,
                    reason: Some("NO_BROKER_CONNECTED".to_string()),
                    ts: chrono::Utc::now(),
                };
                let order_id = ctx.store.insert_order(order).await?;
                ctx.store.record_execution(order_id, &rej).await?;
                metrics::inc_labeled("orders_rejected_total", "reason", "no_broker".into());
                ctx.bus.publish(subjects::EXEC_REJECT, &rej).await?;
                return Ok(());
            }
        },
    };
    let mut routed = order.clone();
    routed.broker = broker.clone();
    let order = &routed;

    let exec = if broker == "paper" {
        paper::simulate(order, reference)
    } else {
        execute_live(ctx, order, &broker, reference).await
    };

    // Persist: order row, fill, execution status.
    let order_id = ctx.store.insert_order(order).await?;
    ctx.store.record_execution(order_id, &exec).await?;

    match exec.status {
        ExecStatus::Filled | ExecStatus::Partial => {
            metrics::inc("orders_filled_total");
            update_position(ctx, order, &exec).await?;
            ctx.bus.publish(subjects::EXEC_FILL, &exec).await?;
        }
        ExecStatus::Rejected | ExecStatus::Canceled => {
            metrics::inc_labeled("orders_rejected_total", "reason",
                exec.reason.clone().unwrap_or_else(|| "exec_reject".into()));
            // Buying-power backoff: if the venue rejected an ENTRY for insufficient
            // funds, pause this tenant's new entries briefly so we don't hammer the
            // venue with dozens of doomed orders while cash is unsettled/short. Set
            // a short cooldown the risk service checks before producing entries.
            // Exits are never affected (they don't reach this on a reduce-only).
            let insufficient = exec.reason.as_deref().map(|r| r.contains("INSUFFICIENT_FUNDS")).unwrap_or(false);
            if insufficient && !order.reduce_only {
                let _ = ctx.store.set_cooldown(&format!("fundhalt:{}", order.tenant), 60).await;
            }
            ctx.bus.publish(subjects::EXEC_REJECT, &exec).await?;
        }
    }

    metrics::observe_latency("order_submission_latency_seconds", started.elapsed().as_secs_f64());
    Ok(())
}

/// Live path: route the order to the tenant's *connected* broker (the one they
/// linked in the dashboard). If the tenant has no connected broker, fall back to
/// the paper simulator so their bots still trade (virtually) rather than error.
/// Broker keys are pulled from the tenant's Vault partition into a local value
/// for the single submit and dropped immediately after.
async fn execute_live(ctx: &Context, order: &OrderIntent, broker: &str, reference: f64) -> Execution {
    let outcome = |status: ExecStatus, reason: &str| Execution {
        client_id: order.client_id.clone(),
        tenant: order.tenant,
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        side: order.side,
        status,
        filled_qty: 0.0,
        avg_price: 0.0,
        fee: 0.0,
        exchange_oid: None,
        reason: Some(reason.to_string()),
        ts: chrono::Utc::now(),
    };
    let reject = |reason: &str| outcome(ExecStatus::Rejected, reason);
    // A clean SKIP (Canceled) — recorded for the audit trail but not counted as a
    // failed trade, vs a venue Rejected.
    let skip = |reason: &str| outcome(ExecStatus::Canceled, reason);

    // Min-notional gate: spot venues reject orders below a floor (Alpaca's crypto
    // minimum cost basis is $10). Skip cleanly here instead of loading keys and
    // eating a confusing venue 403 ("cost basis must be >= minimal amount of
    // order 10"). Brokers with no known floor pass straight through.
    //
    // EXITS are exempt: a position can drift below the venue minimum, and it must
    // still be closable — blocking a reduce-only order here would trap the
    // position and prevent stop-losses from ever firing. Let the venue decide.
    let cost_basis = order.qty * order.price.unwrap_or(reference);
    let floor = tcore::brokers::min_notional_usd(broker);
    if !order.reduce_only && floor > 0.0 && cost_basis < floor {
        return skip(&format!(
            "BELOW_MIN_NOTIONAL: ${cost_basis:.2} is below {broker}'s ${floor:.2} minimum order size"
        ));
    }

    let creds = match vault::load_tenant_broker_creds(
        &ctx.config.vault_addr,
        &ctx.config.vault_token,
        order.tenant,
        broker,
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, %broker, "could not load tenant broker creds from vault");
            return reject("VAULT_KEY_LOAD_FAILED");
        }
    };

    // Budget-aware sizing for ENTRIES: the order is already sized within the bot's
    // min/max% of allocation. But if the venue's available cash can't cover it,
    // shrink the order to fit (going BELOW the min% if needed) down to the venue
    // floor — so a small balance still trades instead of erroring. Below the floor
    // there's nothing valid to place, so skip cleanly. Exits are never capped
    // (selling frees cash). Falls back to the original size if balance is unknown.
    let mut submit_order = order.clone();
    if matches!(order.side, Side::Buy) && !order.reduce_only {
        if let Some(avail) = exchanges::available_usd(broker, &creds).await {
            let px = order.price.unwrap_or(reference);
            let want = submit_order.qty * px;
            let usable = avail * 0.995; // keep ~0.5% headroom for fees
            if floor > 0.0 && usable < floor {
                drop(creds);
                return skip(&format!(
                    "INSUFFICIENT_FUNDS: ${avail:.2} available on {broker}, below its ${floor:.2} minimum"
                ));
            }
            if px > 0.0 && want > usable {
                submit_order.qty = (usable / px * 1e6).floor() / 1e6;
                tracing::info!(symbol = %order.symbol, from = want, to = usable, "sized entry down to available cash");
            }
        }
    }

    let ack = exchanges::submit(broker, &creds, &submit_order).await;
    // `creds` drops here — secret no longer resident.
    drop(creds);

    let order = &submit_order; // persist/fill reflect the actually-submitted size

    if ack.accepted {
        Execution {
            client_id: order.client_id.clone(),
            tenant: order.tenant,
            exchange: order.exchange.clone(),
            symbol: order.symbol.clone(),
            side: order.side,
            status: ExecStatus::Filled,
            filled_qty: order.qty,
            avg_price: order.price.unwrap_or(reference),
            fee: 0.0,
            exchange_oid: ack.exchange_oid,
            reason: None,
            ts: chrono::Utc::now(),
        }
    } else {
        reject(&ack.reason.unwrap_or_else(|| "VENUE_REJECT".into()))
    }
}

async fn update_position(ctx: &Context, order: &OrderIntent, exec: &Execution) -> Result<()> {
    // If this fill is OPPOSITE to an open position, it CLOSES the trade and
    // realizes P&L; otherwise it opens (or refreshes) a position on its side.
    let held = ctx.store.open_position(order.tenant, &order.symbol).await.ok().flatten();
    let fill_long = matches!(order.side, Side::Buy);
    let (closed, realized) = match &held {
        // Opposite side → close & realize P&L. Same side or flat → open/refresh.
        Some((hside, hqty, entry)) if (hside.as_str() == "long") != fill_long => {
            let dir = if hside.as_str() == "long" { 1.0 } else { -1.0 };
            let pnl = (exec.avg_price - *entry) * (*hqty).min(exec.filled_qty) * dir;
            (true, pnl)
        }
        _ => (false, 0.0),
    };

    let update = PositionUpdate {
        tenant: order.tenant,
        exchange: order.exchange.clone(),
        symbol: order.symbol.clone(),
        strategy: Some(order.strategy.clone()),
        side: paper::position_side(order),
        qty: if closed { 0.0 } else { exec.filled_qty },
        entry_price: exec.avg_price,
        stop_price: order.stop_price,
        take_profit: order.take_profit,
        leverage: order.leverage,
        realized_pnl: realized,
        unrealized_pnl: 0.0,
        closed,
        ts: chrono::Utc::now(),
    };
    ctx.store.upsert_position(&update).await?;
    ctx.bus.publish(tenant::subj_positions(order.tenant), &update).await?;
    let _ = side_label(order.side); // reserved for per-side metrics
    Ok(())
}

fn side_label(side: Side) -> &'static str {
    match side {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}
