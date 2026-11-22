//! Risk service — the ONLY path that produces an order. Every signal is gated
//! through: kill-switch → tenant-halt → stop-defined → stale-data → max-open →
//! daily-drawdown → AI floor → regional compliance (PDT / SEBI rate cap +
//! broker whitelist) → position sizing. Any failure rejects, fails closed.

pub mod monitor;

use tcore::bus::decode;
use tcore::compliance::{broker_whitelisted, OrderRateLimiter, PdtGuard};
use tcore::store::TradeDecision;
use tcore::tenant::{self, Region, Regime, TenantId};
use tcore::types::{OrderIntent, OrderType, Side, Signal};
use tcore::{metrics, subjects, Context, Result};
use futures::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub async fn run(ctx: Arc<Context>) -> Result<()> {
    tracing::info!(region = ctx.config.region.as_str(), "risk service starting");
    spawn_control_listener(ctx.clone());
    spawn_gauge_loop(ctx.clone());

    // Compliance state is shared across the single consumer task.
    let limiter = Mutex::new(OrderRateLimiter::new(ctx.config.default_max_orders_per_sec));
    let pdt = Mutex::new(PdtGuard::default());

    let mut sub = ctx.bus.subscribe(tenant::ALL_TENANT_SIGNALS).await?;
    while let Some(msg) = sub.next().await {
        let signal: Signal = match decode(&msg.payload) {
            Some(s) => s,
            None => continue,
        };
        let started = Instant::now();
        let (action, reason): (&str, String) = match evaluate(&ctx, &signal, &limiter, &pdt).await {
            Ok(Some(order)) => {
                if let Err(e) = ctx.bus.publish(tenant::subj_orders(order.tenant), &order).await {
                    tracing::warn!(error = %e, "failed to publish order");
                    ("skipped", "publish_failed".to_string())
                } else {
                    metrics::inc("orders_approved_total");
                    let _ = ctx.store.meter(order.tenant, "orders", 1).await;
                    ("traded", "approved".to_string())
                }
            }
            Ok(None) => ("skipped", "no_setup".to_string()),
            Err(r) => {
                reject(&ctx, &signal, r).await;
                ("skipped", r.to_string())
            }
        };
        log_decision(&ctx, &signal, action, &reason).await;
        metrics::observe_latency("risk_latency_seconds", started.elapsed().as_secs_f64());
    }
    Ok(())
}

/// Record why a signal was traded or skipped — the explainability log. Reads
/// the live news sentiment + market regime so the decision is fully explained.
async fn log_decision(ctx: &Context, s: &Signal, action: &str, reason: &str) {
    let coin = s.symbol.trim_end_matches("USDT");
    let sentiment = ctx.store.sentiment(coin).await.ok().flatten().map(|(score, _)| score);
    let regime = ctx.store.current_market_regime().await;
    let decision = TradeDecision {
        tenant: s.tenant,
        symbol: s.symbol.clone(),
        strategy: s.strategy.clone(),
        side: format!("{:?}", s.side).to_lowercase(),
        signal_strength: s.strength,
        ai_confidence: s.ai_score,
        ai_mode: s.ai_score.map(|_| "heuristic".to_string()),
        sentiment,
        regime,
        action: action.to_string(),
        reason: reason.to_string(),
    };
    if let Err(e) = ctx.store.record_decision(&decision).await {
        tracing::warn!(error = %e, "failed to record trade decision");
    }
}

/// Returns Ok(Some(order)) when approved, Err(reason) when rejected.
async fn evaluate(
    ctx: &Context,
    s: &Signal,
    limiter: &Mutex<OrderRateLimiter>,
    pdt: &Mutex<PdtGuard>,
) -> std::result::Result<Option<OrderIntent>, &'static str> {
    let cfg = &ctx.config;
    let tenant = s.tenant;
    let regime = cfg.region.regime();

    // 1. Global kill switch + per-tenant halt.
    if ctx.store.kill_switch().await.unwrap_or(true) {
        return Err("KILL_SWITCH");
    }
    if ctx.store.tenant_halted(tenant).await.unwrap_or(false) {
        return Err("TENANT_HALTED");
    }

    // 2. Defined risk: a stop is mandatory.
    let risk_per_unit = (s.price - s.stop).abs();
    if s.stop <= 0.0 || risk_per_unit <= 0.0 {
        return Err("NO_STOP");
    }

    // 3. Stale-data guard. NOTE: `age` is signed and can be NEGATIVE when the
    // exchange's tick timestamp is a few ms ahead of our clock (normal skew /
    // Binance event time). Compare as i64 — casting a negative age to u64 would
    // wrap to a huge number and falsely reject every signal as STALE_PRICE.
    let now_ms = chrono::Utc::now().timestamp_millis();
    if let Ok(Some(age)) = ctx.store.price_age_ms(&s.exchange, &s.symbol, now_ms).await {
        if age > cfg.stale_ms as i64 {
            return Err("STALE_PRICE");
        }
    }

    // 4. Position state. An OPPOSITE-side signal on an open position is an EXIT
    // (close it, realize P&L) and bypasses entry gates so the bot can always get
    // flat and complete the trade. A SAME-side signal is ignored (no stacking).
    // Otherwise this is a fresh ENTRY.
    let cd_key = format!("{}:{}:{}", tenant, s.strategy, s.symbol);
    let sig_long = matches!(s.side, Side::Buy);
    if let Some((held_side, held_qty, _)) = ctx.store.open_position(tenant, &s.symbol).await.ok().flatten() {
        if (held_side == "long") == sig_long {
            return Err("ALREADY_IN_POSITION");
        }
        // EXIT: close the full held quantity at market.
        return Ok(Some(OrderIntent {
            client_id: format!("{}-{}-x", tenant, s.id),
            tenant,
            strategy: s.strategy.clone(),
            broker: broker_for(cfg.region, &s.exchange),
            exchange: s.exchange.clone(),
            symbol: s.symbol.clone(),
            side: s.side,
            order_type: OrderType::Market,
            qty: held_qty,
            price: Some(s.price),
            stop_price: Some(s.stop),
            take_profit: s.take_profit,
            leverage: 1.0,
            reduce_only: true, // EXIT — must always be allowed through.
            ts: chrono::Utc::now(),
        }));
    }

    // ---- ENTRY path (currently flat in this symbol) ----
    // Resolve the broker this order will actually route to (the same resolution
    // execution uses), so the capability + minimum-order gates below are
    // accurate. Paper mode and tenants with no connected broker → paper sim.
    let effective_broker = match cfg.trading_mode {
        tcore::config::TradingMode::Paper => "paper".to_string(),
        tcore::config::TradingMode::Live => ctx
            .store
            .bot_broker_for_strategy(tenant, &s.strategy)
            .await
            .ok()
            .flatten()
            .filter(|b| b != "paper")
            .unwrap_or_else(|| "paper".to_string()),
    };

    // 4·short. Broker capability: spot-only venues (e.g. Alpaca crypto) cannot
    // open a SHORT. Block the short ENTRY here so the engine never tracks a
    // position the venue can't hold — and therefore can never close (the cause
    // of the phantom-short exit-order floods). Exits of held positions are
    // handled above and unaffected; the paper simulator and short-capable /
    // unknown brokers pass through, so other brokers never break.
    if matches!(s.side, Side::Sell) && !tcore::brokers::supports_crypto_short(&effective_broker) {
        return Err("SHORT_NOT_SUPPORTED");
    }

    // 4·funds. Buying-power backoff: execution sets this cooldown after the venue
    // rejects an entry for insufficient funds (e.g. unsettled cash). While active,
    // skip producing new entries instead of firing dozens of doomed orders. Auto-
    // expires (~60s) so it resumes the moment funds are usable. Exits are handled
    // above and never reach here, so closing is never blocked.
    if ctx.store.in_cooldown(&format!("fundhalt:{}", tenant)).await.unwrap_or(false) {
        return Err("INSUFFICIENT_FUNDS_COOLDOWN");
    }

    // 4a. Max concurrent positions for this tenant.
    let open = ctx.store.open_position_count(tenant).await.unwrap_or(i64::MAX);
    if open >= cfg.risk.max_open_positions as i64 {
        return Err("MAX_OPEN_POSITIONS");
    }
    // 4b. Per-symbol cooldown — pace re-entries (no firehose).
    if ctx.store.in_cooldown(&cd_key).await.unwrap_or(false) {
        return Err("COOLDOWN");
    }

    // 5. Daily-drawdown circuit breaker → halts the tenant.
    let equity = ctx.store.equity_usd(tenant, cfg.starting_equity_usd).await.unwrap_or(cfg.starting_equity_usd);
    let realized_today = ctx.store.realized_pnl_today(tenant).await.unwrap_or(0.0);
    let dd_pct = if equity > 0.0 { (-realized_today / equity) * 100.0 } else { 0.0 };
    metrics::gauge("daily_drawdown_pct", dd_pct.max(0.0));
    if dd_pct >= cfg.risk.daily_drawdown_pct {
        let _ = ctx.store.set_tenant_halt(tenant, true).await;
        let _ = ctx.store.audit(tenant, "system", "daily_drawdown_halt", None).await;
        return Err("DAILY_DRAWDOWN");
    }

    // 6. AI advisory floor (only when a score is present).
    let mut ai_floor = 0.25;

    // 6b. Macro event-window awareness: tighten conviction near CPI/FOMC/NFP.
    if ctx.store.macro_event_soon().await.unwrap_or(false) {
        ai_floor = 0.40;
        metrics::inc("risk_macro_window_total");
    }
    if let Some(score) = s.ai_score {
        if score < ai_floor {
            return Err("AI_SCORE_FLOOR");
        }
    }

    // 6c. News sentiment veto (advisory, read-only). Strong, confident news that
    // contradicts the trade direction blocks the entry — it never creates one.
    let asset = base_asset(&s.symbol);
    if let Ok(Some((sent, conf))) = ctx.store.sentiment(&asset).await {
        metrics::gauge("news_sentiment_score", sent);
        let contradicts = match s.side {
            tcore::types::Side::Buy => sent <= -40.0,
            tcore::types::Side::Sell => sent >= 40.0,
        };
        if conf >= 60.0 && contradicts {
            return Err("NEWS_CONTRADICTS");
        }
    }

    // 7. Regional compliance.
    let broker = broker_for(cfg.region, &s.exchange);
    broker_whitelisted(regime, &broker).map_err(|d| d.reason())?;
    if regime == Regime::Finra {
        let mut g = pdt.lock().await;
        g.check(tenant, equity, Instant::now()).map_err(|d| d.reason())?;
        g.record(tenant, Instant::now());
    }
    {
        let mut l = limiter.lock().await;
        if !l.allow(tenant, Instant::now()) {
            metrics::inc_labeled("orders_rate_limited_total", "region", cfg.region.as_str().to_string());
            return Err("ORDER_RATE_EXCEEDED");
        }
    }

    // 8. Position sizing. Budget is the BOT's allocation (its capital), not the
    // whole account — so a $2k bot never fires a $250k order. Each position is
    // capped to an equal slice of the budget across max open positions, so one
    // bot scanning many markets can't exceed its allocation. Spot = no leverage.
    // The bot's allocation/markets come from its running config. If there is no
    // valid allocation, DO NOT fall back to full account equity — that produced
    // erroneous huge orders (e.g. a $5k order from a $70 bot). Skip until the bot
    // is properly configured.
    let Some((budget, markets, min_pct, max_pct)) = ctx
        .store
        .bot_sizing(tenant, &s.strategy)
        .await
        .ok()
        .flatten()
    else {
        return Err("NO_ALLOCATION");
    };
    // Hard allocation cap: total capital deployed in this strategy's open
    // positions may never exceed the bot's budget.
    let deployed = ctx.store.deployed_capital(tenant, &s.strategy).await.unwrap_or(0.0);
    let headroom = (budget - deployed).max(0.0);
    if headroom < budget * 0.02 {
        // Alert the user once (deduped 6h) that the bot is fully allocated.
        let body = format!(
            "Your {} strategy has fully deployed its ${:.0} allocation (~${:.0} in open positions). \
             New trades are paused until capital frees up — increase the allocation or close positions to resume.",
            s.strategy, budget, deployed
        );
        let dedup = format!("alloc_full:{}:{}", tenant, s.strategy);
        let _ = ctx
            .store
            .notify_once(tenant, "allocation_full", "Allocation limit reached", &body, "warning", &dedup, 21_600)
            .await;
        return Err("ALLOCATION_FULL");
    }

    // AI-decided position size. Each position is min%..max% of the bot's BUDGET
    // (the user's allocation), scaled by AI conviction — so a $70 budget with
    // 10–15% bounds places a real ~$7–$10.5 trade, NOT a token $1. Previously the
    // %-was applied to a per-market slice (budget ÷ markets), which shrank trades
    // to cents and floored them to the venue minimum. Total exposure is still
    // capped by headroom (never exceeds budget); `max_open_positions` limits how
    // many run concurrently, and max% caps any single position.
    let _ = markets;
    let conviction = s.ai_score.unwrap_or(0.5).clamp(0.0, 1.0);
    let lo = budget * (min_pct / 100.0);
    let hi = budget * (max_pct / 100.0);
    let ai_notional = lo + (hi - lo) * conviction;
    let mut notional = ai_notional.min(headroom);

    // Broker minimum-order awareness. A notional below the venue floor (e.g.
    // Alpaca's $10 crypto minimum) can never execute — it would be skipped at
    // submit, so the bot would appear "running" yet never trade. If the bot can
    // still afford a minimum-sized order, round UP to the floor so small
    // allocations actually trade (fewer concurrent markets, but real orders),
    // bounded by headroom. Only when even one minimum order won't fit the
    // remaining budget do we stop — and tell the user why. Brokers with no known
    // floor (paper sim, others) are unaffected.
    let broker_min = tcore::brokers::min_notional_usd(&effective_broker);
    if broker_min > 0.0 && notional < broker_min {
        if headroom >= broker_min {
            notional = broker_min;
        } else {
            let body = format!(
                "Your {} bot's ${:.0} allocation is too small to place an order on {}, \
                 which requires a minimum of ${:.0} per trade. Increase the allocation \
                 (or watch fewer markets) so each trade clears the ${:.0} minimum.",
                s.strategy, budget, effective_broker, broker_min, broker_min
            );
            let dedup = format!("alloc_below_min:{}:{}", tenant, s.strategy);
            let _ = ctx
                .store
                .notify_once(tenant, "allocation_below_min", "Allocation below broker minimum", &body, "warning", &dedup, 21_600)
                .await;
            return Err("BELOW_MIN_ORDER_SIZE");
        }
    }

    let qty = notional / s.price;
    if qty <= 0.0 || !qty.is_finite() {
        return Err("ZERO_QTY");
    }
    // Risk_per_unit feeds the mandatory-stop check above; keep it referenced.
    let _ = risk_per_unit;

    // Approved entry — start this symbol's cooldown so it won't re-fire next candle.
    let cooldown_secs: u64 = std::env::var("ORDER_COOLDOWN_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(45);
    let _ = ctx.store.set_cooldown(&cd_key, cooldown_secs).await;

    let order = OrderIntent {
        client_id: format!("{}-{}", tenant, s.id),
        tenant,
        strategy: s.strategy.clone(),
        broker,
        exchange: s.exchange.clone(),
        symbol: s.symbol.clone(),
        side: s.side,
        order_type: OrderType::Market,
        qty,
        price: Some(s.price),
        stop_price: Some(s.stop),
        take_profit: s.take_profit,
        leverage: 1.0,
        reduce_only: false, // fresh ENTRY.
        ts: chrono::Utc::now(),
    };
    Ok(Some(order))
}

/// Strip a common quote suffix to get the base asset (BTCUSDT -> BTC) so we can
/// look up news sentiment, which is keyed by asset.
fn base_asset(symbol: &str) -> String {
    for q in ["USDT", "USDC", "BUSD", "USD", "FDUSD"] {
        if let Some(base) = symbol.strip_suffix(q) {
            if !base.is_empty() {
                return base.to_string();
            }
        }
    }
    symbol.to_string()
}

/// Map a market-data venue to the broker that will route the order. For crypto
/// the venue *is* the broker; India equities route through empanelled brokers.
fn broker_for(region: Region, exchange: &str) -> String {
    match region {
        Region::ApSouth1 => std::env::var("SEBI_BROKER").unwrap_or_else(|_| "zerodha".to_string()),
        Region::UsEast1 => exchange.to_string(),
    }
}

async fn reject(ctx: &Context, s: &Signal, reason: &'static str) {
    metrics::inc_labeled("orders_rejected_total", "reason", reason.to_string());
    tracing::warn!(tenant = %s.tenant, symbol = %s.symbol, strategy = %s.strategy, reason, "signal rejected");
    let _ = ctx
        .bus
        .publish(subjects::EXEC_REJECT, &serde_json::json!({
            "tenant": s.tenant, "symbol": s.symbol, "strategy": s.strategy, "reason": reason
        }))
        .await;
}

/// Listen for operator control messages (kill-switch, tenant pause).
fn spawn_control_listener(ctx: Arc<Context>) {
    tokio::spawn(async move {
        let mut sub = match ctx.bus.subscribe(subjects::CONTROL_ALL).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "control subscribe failed");
                return;
            }
        };
        while let Some(msg) = sub.next().await {
            if let Some(ctrl) = decode::<tcore::types::Control>(&msg.payload) {
                match ctrl {
                    tcore::types::Control::KillSwitch { on, actor } => {
                        let _ = ctx.store.set_kill_switch(on).await;
                        let _ = ctx.store.audit(TenantId::default_tenant(), &actor,
                            if on { "kill_switch_on" } else { "kill_switch_off" }, None).await;
                        metrics::gauge("kill_switch_engaged", if on { 1.0 } else { 0.0 });
                        tracing::warn!(on, %actor, "kill switch toggled");
                    }
                    tcore::types::Control::Strategy { name, enabled, actor } => {
                        tracing::info!(%name, enabled, %actor, "strategy toggle (applies on restart)");
                    }
                }
            }
        }
    });
}

/// Periodically refresh slow-moving gauges for Prometheus/Grafana.
fn spawn_gauge_loop(ctx: Arc<Context>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            tick.tick().await;
            let ks = ctx.store.kill_switch().await.unwrap_or(false);
            metrics::gauge("kill_switch_engaged", if ks { 1.0 } else { 0.0 });
            metrics::gauge("risk_daily_drawdown_limit_pct", ctx.config.risk.daily_drawdown_pct);
            if let Ok(n) = ctx.store.open_position_count(TenantId::default_tenant()).await {
                metrics::gauge("open_positions", n as f64);
            }
        }
    });
}
