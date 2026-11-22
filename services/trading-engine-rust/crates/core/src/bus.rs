//! Thin JSON-over-NATS helpers. The hot path publishes/consumes typed events
//! without re-implementing (de)serialisation everywhere.

use anyhow::Result;
use async_nats::{Client, Subscriber};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Clone)]
pub struct Bus {
    client: Client,
}

impl Bus {
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url).await?;
        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Serialise `value` to JSON and publish on `subject` (accepts `&str` or
    /// `String`).
    pub async fn publish<T: Serialize>(&self, subject: impl Into<String>, value: &T) -> Result<()> {
        let payload = serde_json::to_vec(value)?;
        self.client.publish(subject.into(), payload.into()).await?;
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<Subscriber> {
        Ok(self.client.subscribe(subject.to_string()).await?)
    }

    pub async fn flush(&self) -> Result<()> {
        self.client.flush().await?;
        Ok(())
    }
}

/// Decode a NATS message payload as JSON `T`, logging and skipping on error.
pub fn decode<T: DeserializeOwned>(payload: &[u8]) -> Option<T> {
    match serde_json::from_slice::<T>(payload) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!(error = %e, "failed to decode message payload");
            None
        }
    }
}
