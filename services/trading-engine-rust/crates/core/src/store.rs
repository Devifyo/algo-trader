//! Stateful stores: Redis (hot state, prices, kill-switch) and Postgres /
//! TimescaleDB (durable history). Methods clone the multiplexed Redis handle
//! per call — cheap (Arc inside) and avoids holding a lock on the hot path.

use crate::tenant::TenantId;
use crate::types::{Candle, Execution, OrderIntent, PositionUpdate, Side};
use anyhow::Result;
use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Store {
    redis: redis::aio::MultiplexedConnection,
    pg: PgPool,
}

/// One market's live opportunity score + factor breakdown (scanner output).
#[derive(Clone, Debug)]
pub struct MarketScore {
    pub symbol: String,
    pub market_score: f64,
    pub bullish: f64,
    pub bearish: f64,
    pub confidence: f64,
    pub risk: f64,
    pub trend: f64,
    pub momentum: f64,
    pub volatility: f64,
    pub sentiment: f64,
    pub price: f64,
    pub bias: String,
}

/// One AI/risk decision for the explainability log (why a trade ran or didn't).
#[derive(Clone, Debug)]
pub struct TradeDecision {
    pub tenant: TenantId,
    pub symbol: String,
    pub strategy: String,
    pub side: String,
    pub signal_strength: f64,
    pub ai_confidence: Option<f64>,
    pub ai_mode: Option<String>,
    pub sentiment: Option<f64>,
    pub regime: Option<String>,
    pub action: String,
    pub reason: String,
}

impl Store {
    pub async fn connect(redis_url: &str, database_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let redis = client.get_multiplexed_async_connection().await?;
        let pg = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { redis, pg })
    }

    pub fn pg(&self) -> &PgPool {
        &self.pg
    }

    // ── Hot state in Redis ──────────────────────────────────────────────
    pub async fn set_price(&self, exchange: &str, symbol: &str, price: f64, ts_ms: i64) -> Result<()> {
        let mut con = self.redis.clone();
        let key = format!("px:{exchange}:{symbol}");
        // Store price and last-update timestamp; TTL guards against stale reads.
        let _: () = con.set_ex(&key, price, 60).await?;
        let _: () = con.set_ex(format!("px_ts:{exchange}:{symbol}"), ts_ms, 60).await?;
        Ok(())
    }

    pub async fn get_price(&self, exchange: &str, symbol: &str) -> Result<Option<f64>> {
        let mut con = self.redis.clone();
        let v: Option<f64> = con.get(format!("px:{exchange}:{symbol}")).await?;
        Ok(v)
    }

    /// The broker this tenant has connected (most recent safe connection), e.g.
    /// "alpaca". Returns None when the tenant has linked no broker.
    pub async fn tenant_broker(&self, tenant: TenantId) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT broker_id FROM exchange_keys
             WHERE tenant_id = $1 AND safe = true AND broker_id IS NOT NULL
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .fetch_optional(&self.pg)
        .await?;
        Ok(row.map(|r| r.0))
    }

    /// The capital budget (USD) of a running bot for `(tenant, strategy)`, read
    /// from its config `alloc_total`. Used to bound position size to the bot's
    /// allocation rather than the whole account.
    pub async fn bot_allocation_for_strategy(&self, tenant: TenantId, strategy: &str) -> Result<Option<f64>> {
        let row: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            "SELECT config FROM bots
             WHERE tenant_id = $1 AND strategy = $2 AND status = 'running'
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(strategy)
        .fetch_optional(&self.pg)
        .await?;
        Ok(row
            .and_then(|(c,)| c)
            .and_then(|v| v.get("alloc_total").and_then(|x| x.as_f64()))
            .filter(|a| *a > 0.0))
    }

    /// Sizing config for a running bot: (allocation, market_count, min_pct, max_pct).
    /// min/max_pct bound how much of the per-market budget a single AI-sized trade
    /// may use. Defaults: 25%..100%.
    pub async fn bot_sizing(&self, tenant: TenantId, strategy: &str) -> Result<Option<(f64, i64, f64, f64)>> {
        let row: Option<(Option<serde_json::Value>,)> = sqlx::query_as(
            "SELECT config FROM bots
             WHERE tenant_id = $1 AND strategy = $2 AND status = 'running'
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(strategy)
        .fetch_optional(&self.pg)
        .await?;
        Ok(row.and_then(|(c,)| c).and_then(|v| {
            let alloc = v.get("alloc_total").and_then(|x| x.as_f64()).unwrap_or(0.0);
            if alloc <= 0.0 {
                return None;
            }
            let markets = v.get("watchlist").and_then(|w| w.as_array()).map(|a| a.len() as i64).unwrap_or(1).max(1);
            let min_pct = v.get("min_pct").and_then(|x| x.as_f64()).unwrap_or(25.0).clamp(1.0, 100.0);
            let max_pct = v.get("max_pct").and_then(|x| x.as_f64()).unwrap_or(100.0).clamp(min_pct, 100.0);
            Some((alloc, markets, min_pct, max_pct))
        }))
    }

    /// The open position in `symbol` for a tenant: (side "long"|"short", qty, entry_price).
    pub async fn open_position(&self, tenant: TenantId, symbol: &str) -> Result<Option<(String, f64, f64)>> {
        let row: Option<(String, f64, f64)> = sqlx::query_as(
            "SELECT side, qty, entry_price FROM positions
             WHERE status = 'open' AND tenant_id = $1 AND symbol = $2 LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(symbol)
        .fetch_optional(&self.pg)
        .await?;
        Ok(row)
    }

    /// Every open position across tenants — for the stop/trailing monitor.
    /// Returns (id, tenant_id, exchange, symbol, strategy, side, qty, entry, stop, take_profit).
    #[allow(clippy::type_complexity)]
    pub async fn all_open_positions(
        &self,
    ) -> Result<Vec<(uuid::Uuid, uuid::Uuid, String, String, String, String, f64, f64, Option<f64>, Option<f64>)>> {
        let rows = sqlx::query_as(
            "SELECT id, tenant_id, exchange, symbol, strategy, side, qty, entry_price, stop_price, take_profit
             FROM positions WHERE status = 'open'",
        )
        .fetch_all(&self.pg)
        .await?;
        Ok(rows)
    }

    /// Ratchet a position's protective stop (trailing). Only moves it; never widens.
    pub async fn update_position_stop(&self, id: uuid::Uuid, stop: f64) -> Result<()> {
        sqlx::query("UPDATE positions SET stop_price = $2 WHERE id = $1 AND status = 'open'")
            .bind(id)
            .bind(stop)
            .execute(&self.pg)
            .await?;
        Ok(())
    }

    /// How many markets a running bot for `(tenant, strategy)` watches (1 for a
    /// single-symbol bot, N for a watchlist, the full universe for AI-auto). Used
    /// to split the allocation across markets when sizing positions.
    pub async fn bot_market_count(&self, tenant: TenantId, strategy: &str) -> Result<Option<i64>> {
        let row: Option<(Option<i64>,)> = sqlx::query_as(
            "SELECT jsonb_array_length(config->'watchlist') FROM bots
             WHERE tenant_id = $1 AND strategy = $2 AND status = 'running'
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(strategy)
        .fetch_optional(&self.pg)
        .await?;
        Ok(row.and_then(|(n,)| n).filter(|n| *n > 0))
    }

    /// The broker a running bot for `(tenant, strategy)` is attached to, e.g.
    /// "alpaca" or "paper". None when no running bot matches.
    pub async fn bot_broker_for_strategy(&self, tenant: TenantId, strategy: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT exchange FROM bots
             WHERE tenant_id = $1 AND strategy = $2 AND status = 'running' AND exchange IS NOT NULL
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(strategy)
        .fetch_optional(&self.pg)
        .await?;
        Ok(row.map(|r| r.0))
    }

    // ──────────────── Market intelligence (opportunity scanner) ────────────────

    /// Running bots → (tenant, strategy, config_json, broker). The source of
    /// truth for what the engine trades, on which markets, and via which broker.
    pub async fn active_bots(&self) -> Result<Vec<(uuid::Uuid, String, String, String)>> {
        let rows: Vec<(uuid::Uuid, String, Option<serde_json::Value>, Option<String>)> = sqlx::query_as(
            "SELECT tenant_id, strategy, config, exchange FROM bots WHERE status = 'running'",
        )
        .fetch_all(&self.pg)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(t, s, c, b)| (t, s, c.map(|v| v.to_string()).unwrap_or_default(), b.unwrap_or_else(|| "paper".into())))
            .collect())
    }

    pub async fn upsert_market_score(&self, s: &MarketScore) -> Result<()> {
        sqlx::query(
            "INSERT INTO market_scores
               (symbol,market_score,bullish_score,bearish_score,confidence_score,risk_score,
                trend_strength,momentum,volatility,sentiment,price,bias,updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,now())
             ON CONFLICT (symbol) DO UPDATE SET
               market_score=$2,bullish_score=$3,bearish_score=$4,confidence_score=$5,risk_score=$6,
               trend_strength=$7,momentum=$8,volatility=$9,sentiment=$10,price=$11,bias=$12,updated_at=now()",
        )
        .bind(&s.symbol).bind(s.market_score).bind(s.bullish).bind(s.bearish).bind(s.confidence)
        .bind(s.risk).bind(s.trend).bind(s.momentum).bind(s.volatility).bind(s.sentiment)
        .bind(s.price).bind(&s.bias)
        .execute(&self.pg).await?;
        Ok(())
    }

    pub async fn set_market_regime(&self, regime: &str, volatility: &str, breadth: f64, avg_vol: f64, favored: &[String]) -> Result<()> {
        sqlx::query(
            "INSERT INTO market_regime (id,regime,volatility,breadth,avg_volatility,favored,updated_at)
             VALUES (1,$1,$2,$3,$4,$5,now())
             ON CONFLICT (id) DO UPDATE SET regime=$1,volatility=$2,breadth=$3,avg_volatility=$4,favored=$5,updated_at=now()",
        )
        .bind(regime).bind(volatility).bind(breadth).bind(avg_vol).bind(favored)
        .execute(&self.pg).await?;
        Ok(())
    }

    pub async fn set_capital_allocation(&self, rows: &[(String, f64, i32, String)]) -> Result<()> {
        let mut tx = self.pg.begin().await?;
        sqlx::query("DELETE FROM capital_allocation").execute(&mut *tx).await?;
        for (sym, w, rank, reason) in rows {
            sqlx::query("INSERT INTO capital_allocation (symbol,weight,rank,reason,updated_at) VALUES ($1,$2,$3,$4,now())")
                .bind(sym).bind(w).bind(rank).bind(reason)
                .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// The latest market regime label (from the scanner), for decision logging.
    pub async fn current_market_regime(&self) -> Option<String> {
        sqlx::query_as::<_, (String,)>("SELECT regime FROM market_regime WHERE id = 1")
            .fetch_optional(&self.pg)
            .await
            .ok()
            .flatten()
            .map(|r| r.0)
    }

    /// Append one decision to the explainability log.
    pub async fn record_decision(&self, d: &TradeDecision) -> Result<()> {
        sqlx::query(
            "INSERT INTO trade_decisions
               (tenant_id,symbol,strategy,side,signal_strength,ai_confidence,ai_mode,sentiment,regime,action,reason)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)",
        )
        .bind(d.tenant.as_uuid()).bind(&d.symbol).bind(&d.strategy).bind(&d.side)
        .bind(d.signal_strength).bind(d.ai_confidence).bind(&d.ai_mode).bind(d.sentiment)
        .bind(&d.regime).bind(&d.action).bind(&d.reason)
        .execute(&self.pg).await?;
        Ok(())
    }

    pub async fn price_age_ms(&self, exchange: &str, symbol: &str, now_ms: i64) -> Result<Option<i64>> {
        let mut con = self.redis.clone();
        let ts: Option<i64> = con.get(format!("px_ts:{exchange}:{symbol}")).await?;
        Ok(ts.map(|t| now_ms - t))
    }

    pub async fn kill_switch(&self) -> Result<bool> {
        let mut con = self.redis.clone();
        let v: Option<String> = con.get("kill_switch").await?;
        Ok(matches!(v.as_deref(), Some("1") | Some("on") | Some("true")))
    }

    pub async fn set_kill_switch(&self, on: bool) -> Result<()> {
        let mut con = self.redis.clone();
        let _: () = con.set("kill_switch", if on { "1" } else { "0" }).await?;
        Ok(())
    }

    /// Per-tenant trading halt (drawdown breaker, suspension, operator pause).
    pub async fn tenant_halted(&self, tenant: TenantId) -> Result<bool> {
        let mut con = self.redis.clone();
        let v: Option<String> = con.get(format!("halt:{tenant}")).await?;
        Ok(matches!(v.as_deref(), Some("1") | Some("on") | Some("true")))
    }

    pub async fn set_tenant_halt(&self, tenant: TenantId, on: bool) -> Result<()> {
        let mut con = self.redis.clone();
        if on {
            let _: () = con.set(format!("halt:{tenant}"), "1").await?;
        } else {
            let _: () = con.del(format!("halt:{tenant}")).await?;
        }
        Ok(())
    }

    /// Advisory news sentiment for an asset (e.g. "BTC"), written by the news
    /// service. Returns `(score -100..100, confidence 0..100)`. Read-only here:
    /// the risk engine may tighten or veto on it but never trades on it.
    pub async fn sentiment(&self, asset: &str) -> Result<Option<(f64, f64)>> {
        let mut con = self.redis.clone();
        let raw: Option<String> = con.get(format!("sentiment:{asset}")).await?;
        let Some(raw) = raw else { return Ok(None) };
        #[derive(serde::Deserialize)]
        struct S { score: f64, confidence: f64 }
        match serde_json::from_str::<S>(&raw) {
            Ok(s) => Ok(Some((s.score, s.confidence))),
            Err(_) => Ok(None),
        }
    }

    /// True when a high-impact macro event is imminent (set by the news
    /// service economic calendar). The risk engine reduces conviction in-window.
    pub async fn macro_event_soon(&self) -> Result<bool> {
        let mut con = self.redis.clone();
        let v: Option<String> = con.get("macro:event_soon").await?;
        Ok(v.is_some())
    }

    pub async fn open_position_count(&self, tenant: TenantId) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM positions WHERE status = 'open' AND tenant_id = $1",
        )
        .bind(tenant.as_uuid())
        .fetch_one(&self.pg)
        .await?;
        Ok(row.0)
    }

    /// USD capital currently deployed in this strategy's open positions — used to
    /// cap total exposure to the bot's allocation.
    pub async fn deployed_capital(&self, tenant: TenantId, strategy: &str) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT sum(qty * entry_price) FROM positions
             WHERE status = 'open' AND tenant_id = $1 AND strategy = $2",
        )
        .bind(tenant.as_uuid())
        .bind(strategy)
        .fetch_one(&self.pg)
        .await?;
        Ok(row.0.unwrap_or(0.0))
    }

    /// Is the tenant already holding an open position in this symbol? Prevents
    /// stacking the same trade every candle.
    pub async fn has_open_position(&self, tenant: TenantId, symbol: &str) -> Result<bool> {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM positions WHERE status = 'open' AND tenant_id = $1 AND symbol = $2",
        )
        .bind(tenant.as_uuid())
        .bind(symbol)
        .fetch_one(&self.pg)
        .await?;
        Ok(row.0 > 0)
    }

    /// Per-key order cooldown (Redis TTL) so a strategy can't re-fire the same
    /// symbol every candle — caps order churn regardless of fills/rejections.
    pub async fn in_cooldown(&self, key: &str) -> Result<bool> {
        let mut con = self.redis.clone();
        let v: Option<String> = con.get(format!("cooldown:{key}")).await?;
        Ok(v.is_some())
    }
    pub async fn set_cooldown(&self, key: &str, secs: u64) -> Result<()> {
        let mut con = self.redis.clone();
        let _: () = con.set_ex(format!("cooldown:{key}"), "1", secs).await?;
        Ok(())
    }

    /// Write a notification for the tenant, de-duplicated by `dedup_key` for
    /// `ttl_secs` (so an event that recurs every candle alerts only once).
    pub async fn notify_once(
        &self, tenant: TenantId, kind: &str, title: &str, body: &str,
        severity: &str, dedup_key: &str, ttl_secs: u64,
    ) -> Result<()> {
        let mut con = self.redis.clone();
        let flag = format!("notified:{dedup_key}");
        let seen: Option<String> = con.get(&flag).await?;
        if seen.is_some() {
            return Ok(());
        }
        let _: () = con.set_ex(&flag, "1", ttl_secs).await?;
        sqlx::query(
            "INSERT INTO notifications (tenant_id,kind,title,body,severity) VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(tenant.as_uuid()).bind(kind).bind(title).bind(body).bind(severity)
        .execute(&self.pg)
        .await?;
        Ok(())
    }

    pub async fn equity_usd(&self, tenant: TenantId, fallback: f64) -> Result<f64> {
        let row: Option<(f64,)> = sqlx::query_as(
            "SELECT equity_usd FROM pnl_snapshots WHERE tenant_id = $1 ORDER BY time DESC LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .fetch_optional(&self.pg)
        .await?;
        Ok(row.map(|r| r.0).unwrap_or(fallback))
    }

    /// Sum of today's realized PnL for a tenant — used by the daily-drawdown
    /// circuit breaker.
    pub async fn realized_pnl_today(&self, tenant: TenantId) -> Result<f64> {
        let row: (Option<f64>,) = sqlx::query_as(
            "SELECT sum(realized_pnl) FROM positions
             WHERE tenant_id = $1 AND status = 'closed' AND closed_at >= date_trunc('day', now())",
        )
        .bind(tenant.as_uuid())
        .fetch_one(&self.pg)
        .await?;
        Ok(row.0.unwrap_or(0.0))
    }

    // ── Durable writes to Timescale/Postgres ────────────────────────────
    pub async fn insert_candle(&self, c: &Candle) -> Result<()> {
        sqlx::query(
            "INSERT INTO candles (time,exchange,symbol,timeframe,open,high,low,close,volume)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
             ON CONFLICT (exchange,symbol,timeframe,time) DO UPDATE
               SET open=$5, high=$6, low=$7, close=$8, volume=$9",
        )
        .bind(c.ts)
        .bind(&c.exchange)
        .bind(&c.symbol)
        .bind(&c.timeframe)
        .bind(c.open)
        .bind(c.high)
        .bind(c.low)
        .bind(c.close)
        .bind(c.volume)
        .execute(&self.pg)
        .await?;
        Ok(())
    }

    /// Persist an order intent (status = submitted). Returns the order UUID.
    pub async fn insert_order(&self, o: &OrderIntent) -> Result<uuid::Uuid> {
        let id: (uuid::Uuid,) = sqlx::query_as(
            "INSERT INTO orders (tenant_id,client_id,exchange,broker,symbol,strategy,side,type,qty,price,stop_price,status)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'submitted')
             ON CONFLICT (client_id) DO UPDATE SET status = orders.status
             RETURNING id",
        )
        .bind(o.tenant.as_uuid())
        .bind(&o.client_id)
        .bind(&o.exchange)
        .bind(&o.broker)
        .bind(&o.symbol)
        .bind(&o.strategy)
        .bind(format!("{:?}", o.side).to_lowercase())
        .bind(format!("{:?}", o.order_type).to_lowercase())
        .bind(o.qty)
        .bind(o.price)
        .bind(o.stop_price)
        .fetch_one(&self.pg)
        .await?;
        Ok(id.0)
    }

    pub async fn record_execution(&self, order_id: uuid::Uuid, e: &Execution) -> Result<()> {
        let status = format!("{:?}", e.status).to_lowercase();
        sqlx::query("UPDATE orders SET status=$1, exchange_oid=$2, reason=$3, updated_at=now() WHERE id=$4")
            .bind(&status)
            .bind(&e.exchange_oid)
            .bind(&e.reason)
            .bind(order_id)
            .execute(&self.pg)
            .await?;

        if e.filled_qty > 0.0 {
            sqlx::query(
                "INSERT INTO fills (tenant_id,order_id,exchange,symbol,side,qty,price,fee,filled_at)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
            )
            .bind(e.tenant.as_uuid())
            .bind(order_id)
            .bind(&e.exchange)
            .bind(&e.symbol)
            .bind(format!("{:?}", e.side).to_lowercase())
            .bind(e.filled_qty)
            .bind(e.avg_price)
            .bind(e.fee)
            .bind(e.ts)
            .execute(&self.pg)
            .await?;
        }
        Ok(())
    }

    pub async fn upsert_position(&self, p: &PositionUpdate) -> Result<()> {
        let side = match p.side {
            Side::Buy => "long",
            Side::Sell => "short",
        };
        if p.closed {
            sqlx::query(
                "UPDATE positions SET status='closed', qty=0, realized_pnl=$4, closed_at=now()
                 WHERE tenant_id=$1 AND exchange=$2 AND symbol=$3 AND status='open'",
            )
            .bind(p.tenant.as_uuid())
            .bind(&p.exchange)
            .bind(&p.symbol)
            .bind(p.realized_pnl)
            .execute(&self.pg)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO positions (tenant_id,exchange,symbol,strategy,side,qty,entry_price,stop_price,take_profit,leverage,realized_pnl,status)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,'open')
                 ON CONFLICT (tenant_id,exchange,symbol) WHERE status='open'
                 DO UPDATE SET qty=$6, entry_price=$7, stop_price=$8, take_profit=$9, leverage=$10, realized_pnl=$11",
            )
            .bind(p.tenant.as_uuid())
            .bind(&p.exchange)
            .bind(&p.symbol)
            .bind(&p.strategy)
            .bind(side)
            .bind(p.qty)
            .bind(p.entry_price)
            .bind(p.stop_price)
            .bind(p.take_profit)
            .bind(p.leverage)
            .bind(p.realized_pnl)
            .execute(&self.pg)
            .await?;
        }
        Ok(())
    }

    pub async fn audit(&self, tenant: TenantId, actor: &str, action: &str, target: Option<&str>) -> Result<()> {
        sqlx::query("INSERT INTO audit_log (tenant_id,actor,action,target) VALUES ($1,$2,$3,$4)")
            .bind(tenant.as_uuid())
            .bind(actor)
            .bind(action)
            .bind(target)
            .execute(&self.pg)
            .await?;
        Ok(())
    }

    /// Load all enabled (tenant, strategy_name) profiles across the region. The
    /// signals service fans these out so each tenant's strategies run isolated.
    pub async fn enabled_strategy_profiles(&self) -> Result<Vec<(uuid::Uuid, String)>> {
        let rows: Vec<(uuid::Uuid, String)> = sqlx::query_as(
            "SELECT tenant_id, name FROM strategies WHERE enabled = true",
        )
        .fetch_all(&self.pg)
        .await?;
        Ok(rows)
    }

    /// Record a billing/metering event (orders, signals, api_calls).
    pub async fn meter(&self, tenant: TenantId, metric: &str, value: i64) -> Result<()> {
        sqlx::query("INSERT INTO api_usage (tenant_id,metric,value) VALUES ($1,$2,$3)")
            .bind(tenant.as_uuid())
            .bind(metric)
            .bind(value)
            .execute(&self.pg)
            .await?;
        Ok(())
    }
}
