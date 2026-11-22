//! Trading strategies. Each consumes a window of closed candles and may return a
//! `RawSignal` (direction + stop + conviction). Risk sizes it afterwards.
//! A strategy that cannot define a protective stop returns `None` — we never
//! trade undefined risk.

use crate::indicators as ind;
use tcore::types::{Candle, Side};

#[derive(Clone, Copy, Debug)]
pub struct RawSignal {
    pub side: Side,
    pub price: f64,
    pub stop: f64,
    pub take_profit: Option<f64>,
    pub strength: f64,
}

pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn evaluate(&self, candles: &[Candle]) -> Option<RawSignal>;
}

/// EMA crossover with ATR-based stop. Long when fast > slow and price above both.
pub struct TrendFollowing {
    pub fast: usize,
    pub slow: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub adx_min: f64,
}
impl Default for TrendFollowing {
    fn default() -> Self {
        Self { fast: 21, slow: 55, atr_period: 14, atr_stop_mult: 2.0, adx_min: 20.0 }
    }
}
impl Strategy for TrendFollowing {
    fn name(&self) -> &'static str { "trend_following" }
    fn evaluate(&self, c: &[Candle]) -> Option<RawSignal> {
        let closes = ind::closes(c);
        let fast = ind::ema(&closes, self.fast)?;
        let slow = ind::ema(&closes, self.slow)?;
        let atr = ind::atr(c, self.atr_period)?;
        // Regime filter: only trade when a real trend exists (ADX above the floor).
        // In chop (low ADX) EMA crossovers whipsaw — skip them.
        let adx = ind::adx(c, self.atr_period)?;
        if adx < self.adx_min {
            return None;
        }
        let price = *closes.last()?;
        let gap = (fast - slow).abs() / slow;
        let strength = (gap * 20.0).clamp(0.0, 1.0);
        if fast > slow && price > fast {
            Some(RawSignal { side: Side::Buy, price, stop: price - self.atr_stop_mult * atr,
                take_profit: Some(price + 3.0 * self.atr_stop_mult * atr), strength })
        } else if fast < slow && price < fast {
            Some(RawSignal { side: Side::Sell, price, stop: price + self.atr_stop_mult * atr,
                take_profit: Some(price - 3.0 * self.atr_stop_mult * atr), strength })
        } else {
            None
        }
    }
}

/// Rate-of-change momentum over a lookback, ATR stop.
pub struct Momentum {
    pub lookback: usize,
    pub threshold: f64,
    pub atr_period: usize,
}
impl Default for Momentum {
    fn default() -> Self { Self { lookback: 20, threshold: 0.02, atr_period: 14 } }
}
impl Strategy for Momentum {
    fn name(&self) -> &'static str { "momentum" }
    fn evaluate(&self, c: &[Candle]) -> Option<RawSignal> {
        if c.len() <= self.lookback { return None; }
        let closes = ind::closes(c);
        let price = *closes.last()?;
        let past = closes[closes.len() - 1 - self.lookback];
        let roc = (price - past) / past;
        let atr = ind::atr(c, self.atr_period)?;
        if roc.abs() < self.threshold { return None; }
        let strength = (roc.abs() / (self.threshold * 3.0)).clamp(0.0, 1.0);
        if roc > 0.0 {
            Some(RawSignal { side: Side::Buy, price, stop: price - 2.0 * atr, take_profit: None, strength })
        } else {
            Some(RawSignal { side: Side::Sell, price, stop: price + 2.0 * atr, take_profit: None, strength })
        }
    }
}

/// Bollinger + RSI mean reversion. Fade extremes back toward the mean.
pub struct MeanReversion {
    pub bb_period: usize,
    pub bb_std: f64,
    pub rsi_period: usize,
}
impl Default for MeanReversion {
    fn default() -> Self { Self { bb_period: 20, bb_std: 2.0, rsi_period: 14 } }
}
impl Strategy for MeanReversion {
    fn name(&self) -> &'static str { "mean_reversion" }
    fn evaluate(&self, c: &[Candle]) -> Option<RawSignal> {
        let closes = ind::closes(c);
        let price = *closes.last()?;
        let bb = ind::bollinger(&closes, self.bb_period, self.bb_std)?;
        let rsi = ind::rsi(&closes, self.rsi_period)?;
        if price < bb.lower && rsi < 30.0 {
            Some(RawSignal { side: Side::Buy, price, stop: price - (bb.mid - bb.lower),
                take_profit: Some(bb.mid), strength: ((30.0 - rsi) / 30.0).clamp(0.0, 1.0) })
        } else if price > bb.upper && rsi > 70.0 {
            Some(RawSignal { side: Side::Sell, price, stop: price + (bb.upper - bb.mid),
                take_profit: Some(bb.mid), strength: ((rsi - 70.0) / 30.0).clamp(0.0, 1.0) })
        } else {
            None
        }
    }
}

/// Donchian breakout with ATR stop.
pub struct Breakout {
    pub channel: usize,
    pub atr_period: usize,
}
impl Default for Breakout {
    fn default() -> Self { Self { channel: 55, atr_period: 14 } }
}
impl Strategy for Breakout {
    fn name(&self) -> &'static str { "breakout" }
    fn evaluate(&self, c: &[Candle]) -> Option<RawSignal> {
        let (hi, lo) = ind::donchian(c, self.channel)?;
        let atr = ind::atr(c, self.atr_period)?;
        let price = c.last()?.close;
        if price > hi {
            Some(RawSignal { side: Side::Buy, price, stop: price - 2.0 * atr, take_profit: None, strength: 0.7 })
        } else if price < lo {
            Some(RawSignal { side: Side::Sell, price, stop: price + 2.0 * atr, take_profit: None, strength: 0.7 })
        } else {
            None
        }
    }
}

/// Volatility-expansion: trade in the direction of a bar that expands ATR after
/// a squeeze.
pub struct VolExpansion {
    pub atr_period: usize,
    pub squeeze_pct: f64,
}
impl Default for VolExpansion {
    fn default() -> Self { Self { atr_period: 14, squeeze_pct: 0.5 } }
}
impl Strategy for VolExpansion {
    fn name(&self) -> &'static str { "vol_expansion" }
    fn evaluate(&self, c: &[Candle]) -> Option<RawSignal> {
        if c.len() < self.atr_period * 3 { return None; }
        let atr_now = ind::atr(c, self.atr_period)?;
        let atr_prev = ind::atr(&c[..c.len() - self.atr_period], self.atr_period)?;
        let last = c.last()?;
        // Expansion: current ATR meaningfully above the prior window's.
        if atr_now > atr_prev * (1.0 + self.squeeze_pct) {
            let bullish = last.close >= last.open;
            let price = last.close;
            if bullish {
                Some(RawSignal { side: Side::Buy, price, stop: price - 2.0 * atr_now, take_profit: None, strength: 0.6 })
            } else {
                Some(RawSignal { side: Side::Sell, price, stop: price + 2.0 * atr_now, take_profit: None, strength: 0.6 })
            }
        } else {
            None
        }
    }
}

/// Build a strategy instance by name with default params. `stat_arb` requires a
/// pair feed and is intentionally not wired here (see docs/RISK.md).
pub fn build(name: &str) -> Option<Box<dyn Strategy>> {
    match name {
        "trend_following" => Some(Box::new(TrendFollowing::default())),
        "momentum" => Some(Box::new(Momentum::default())),
        "mean_reversion" => Some(Box::new(MeanReversion::default())),
        "breakout" => Some(Box::new(Breakout::default())),
        "vol_expansion" => Some(Box::new(VolExpansion::default())),
        _ => None,
    }
}
