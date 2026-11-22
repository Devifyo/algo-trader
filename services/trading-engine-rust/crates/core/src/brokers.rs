//! Broker capability matrix. The single place that knows what each connected
//! venue supports, so the risk + execution gates stay broker-aware. Adding a new
//! broker here is the only edit needed to teach the engine its limits.
//!
//! Defaults are deliberately PERMISSIVE: an unknown / newly-integrated broker is
//! assumed fully capable and floor-free, so introducing another broker never
//! silently blocks its orders. Only venues with a *known* restriction are
//! constrained.

/// Whether a broker can open a SHORT crypto position. Spot-only venues cannot:
/// a sell with no existing long is rejected by the venue, and the engine would
/// be left tracking a position it can never close. Futures-capable venues and
/// the internal paper simulator can short. Unknown brokers are assumed capable.
pub fn supports_crypto_short(broker: &str) -> bool {
    !matches!(broker.to_ascii_lowercase().as_str(), "alpaca" | "coinbase")
}

/// Minimum order cost-basis (USD notional) a broker will accept. Alpaca's crypto
/// minimum is small in practice (sub-$10 buys fill fine — verified live), so the
/// engine should NOT force orders up to a high floor: that makes them too big to
/// fund when available cash is low. Defaults to $1 (Alpaca's documented crypto
/// notional minimum), overridable via `ALPACA_MIN_NOTIONAL_USD` to tune without a
/// rebuild. `0.0` means no known floor — submit as-is.
pub fn min_notional_usd(broker: &str) -> f64 {
    match broker.to_ascii_lowercase().as_str() {
        "alpaca" => std::env::var("ALPACA_MIN_NOTIONAL_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|v| *v >= 0.0)
            .unwrap_or(1.0),
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_venues_cannot_short_others_can() {
        assert!(!supports_crypto_short("alpaca"));
        assert!(!supports_crypto_short("Alpaca"));
        assert!(!supports_crypto_short("coinbase"));
        // paper simulator + any other/unknown broker can short.
        assert!(supports_crypto_short("paper"));
        assert!(supports_crypto_short("binance"));
        assert!(supports_crypto_short("zerodha"));
    }

    #[test]
    fn only_known_floors_apply() {
        assert_eq!(min_notional_usd("alpaca"), 1.0); // default; ALPACA_MIN_NOTIONAL_USD overrides
        assert_eq!(min_notional_usd("paper"), 0.0);
        assert_eq!(min_notional_usd("binance"), 0.0);
    }
}
