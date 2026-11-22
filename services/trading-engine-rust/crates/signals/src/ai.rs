//! Advisory AI scoring. Calls the Python engine's `/score` with a short timeout.
//! On any failure we return `None` and the signal proceeds on its rule score —
//! AI never blocks the hot path.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ScoreRequest<'a> {
    strategy: &'a str,
    symbol: &'a str,
    side: &'a str,
    price: f64,
    strength: f64,
    closes: Vec<f64>,
}

#[derive(Deserialize)]
struct ScoreResponse {
    score: f64,
}

pub async fn score(
    client: &reqwest::Client,
    base_url: &str,
    strategy: &str,
    symbol: &str,
    side: &str,
    price: f64,
    strength: f64,
    closes: &[f64],
) -> Option<f64> {
    let body = ScoreRequest {
        strategy,
        symbol,
        side,
        price,
        strength,
        // cap payload size
        closes: closes.iter().rev().take(64).rev().copied().collect(),
    };
    let resp = client
        .post(format!("{base_url}/score"))
        .json(&body)
        .timeout(Duration::from_millis(50))
        .send()
        .await
        .ok()?;
    let parsed = resp.json::<ScoreResponse>().await.ok()?;
    Some(parsed.score.clamp(0.0, 1.0))
}
