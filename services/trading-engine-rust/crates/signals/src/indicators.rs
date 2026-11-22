//! Pure technical-indicator math over `&[f64]` / candle slices. No allocation on
//! the hot path beyond small fixed buffers.

use tcore::types::Candle;

pub fn sma(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period || period == 0 {
        return None;
    }
    let slice = &values[values.len() - period..];
    Some(slice.iter().sum::<f64>() / period as f64)
}

pub fn ema(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period || period == 0 {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    // Seed with SMA of the first `period` values, then roll forward.
    let mut ema = values[..period].iter().sum::<f64>() / period as f64;
    for &v in &values[period..] {
        ema = v * k + ema * (1.0 - k);
    }
    Some(ema)
}

pub fn stdev(values: &[f64], period: usize) -> Option<f64> {
    if values.len() < period || period == 0 {
        return None;
    }
    let slice = &values[values.len() - period..];
    let mean = slice.iter().sum::<f64>() / period as f64;
    let var = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / period as f64;
    Some(var.sqrt())
}

/// Wilder's RSI.
pub fn rsi(closes: &[f64], period: usize) -> Option<f64> {
    if closes.len() <= period {
        return None;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for w in closes[closes.len() - period - 1..].windows(2) {
        let diff = w[1] - w[0];
        if diff >= 0.0 {
            gain += diff;
        } else {
            loss -= diff;
        }
    }
    let avg_gain = gain / period as f64;
    let avg_loss = loss / period as f64;
    if avg_loss == 0.0 {
        return Some(100.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - 100.0 / (1.0 + rs))
}

/// Average True Range (simple mean of true ranges over `period`).
pub fn atr(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() <= period {
        return None;
    }
    let mut trs = Vec::with_capacity(period);
    for w in candles[candles.len() - period - 1..].windows(2) {
        let (prev, cur) = (&w[0], &w[1]);
        let tr = (cur.high - cur.low)
            .max((cur.high - prev.close).abs())
            .max((cur.low - prev.close).abs());
        trs.push(tr);
    }
    Some(trs.iter().sum::<f64>() / trs.len() as f64)
}

/// Average Directional Index (Wilder), simplified: trend STRENGTH (0–100),
/// direction-agnostic. ADX > ~20–25 ⇒ a real trend exists; low ADX ⇒ chop
/// (a regime filter that keeps trend strategies out of range-bound noise).
pub fn adx(candles: &[Candle], period: usize) -> Option<f64> {
    let n = candles.len();
    if n < 2 * period + 1 {
        return None;
    }
    let (mut tr, mut pdm, mut ndm) = (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
    for i in 1..n {
        let (p, c) = (&candles[i - 1], &candles[i]);
        let up = c.high - p.high;
        let down = p.low - c.low;
        pdm.push(if up > down && up > 0.0 { up } else { 0.0 });
        ndm.push(if down > up && down > 0.0 { down } else { 0.0 });
        tr.push((c.high - c.low).max((c.high - p.close).abs()).max((c.low - p.close).abs()));
    }
    let len = tr.len();
    if len < 2 * period {
        return None;
    }
    let avg = |v: &[f64], end: usize| -> f64 { v[end - period..end].iter().sum::<f64>() / period as f64 };
    let mut dxs = Vec::with_capacity(period);
    for end in (len - period + 1)..=len {
        let atr = avg(&tr, end);
        if atr <= 0.0 {
            continue;
        }
        let pdi = 100.0 * avg(&pdm, end) / atr;
        let ndi = 100.0 * avg(&ndm, end) / atr;
        let denom = pdi + ndi;
        dxs.push(if denom <= 0.0 { 0.0 } else { 100.0 * (pdi - ndi).abs() / denom });
    }
    if dxs.is_empty() {
        return None;
    }
    Some(dxs.iter().sum::<f64>() / dxs.len() as f64)
}

/// On-Balance Volume: running cumulative volume, +vol when close rises vs the
/// prior close, −vol when it falls, unchanged on an equal close. Returns one OBV
/// value per candle (index-aligned). Requires real volume.
pub fn obv(candles: &[Candle]) -> Vec<f64> {
    let mut out = Vec::with_capacity(candles.len());
    let mut total = 0.0;
    for (i, c) in candles.iter().enumerate() {
        if i > 0 {
            let prev = candles[i - 1].close;
            if c.close > prev {
                total += c.volume;
            } else if c.close < prev {
                total -= c.volume;
            }
        }
        out.push(total);
    }
    out
}

/// Least-squares slope of `values` against their index (0,1,2,…). Sign tells the
/// trend direction over the window (used for the OBV rising/falling check).
pub fn linreg_slope(values: &[f64]) -> f64 {
    let n = values.len();
    if n < 2 {
        return 0.0;
    }
    let nf = n as f64;
    let mean_x = (nf - 1.0) / 2.0;
    let mean_y = values.iter().sum::<f64>() / nf;
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &y) in values.iter().enumerate() {
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

pub struct Bollinger {
    pub mid: f64,
    pub upper: f64,
    pub lower: f64,
}

pub fn bollinger(closes: &[f64], period: usize, mult: f64) -> Option<Bollinger> {
    let mid = sma(closes, period)?;
    let sd = stdev(closes, period)?;
    Some(Bollinger {
        mid,
        upper: mid + mult * sd,
        lower: mid - mult * sd,
    })
}

/// Donchian channel high/low over `period` (excludes the current bar).
pub fn donchian(candles: &[Candle], period: usize) -> Option<(f64, f64)> {
    if candles.len() <= period {
        return None;
    }
    let slice = &candles[candles.len() - period - 1..candles.len() - 1];
    let high = slice.iter().fold(f64::MIN, |a, c| a.max(c.high));
    let low = slice.iter().fold(f64::MAX, |a, c| a.min(c.low));
    Some((high, low))
}

pub fn closes(candles: &[Candle]) -> Vec<f64> {
    candles.iter().map(|c| c.close).collect()
}
