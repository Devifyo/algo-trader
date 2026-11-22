//! Minimal Vault KV-v2 client for loading exchange API credentials at boot.
//! Keys NEVER come from env in production — only from here. See docs/SECURITY.md.

use anyhow::Result;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ExchangeCreds {
    pub api_key: String,
    pub api_secret: String,
    #[serde(default)]
    pub passphrase: Option<String>,
    /// "paper" | "live" — for brokers that expose a paper/sandbox environment.
    #[serde(default)]
    pub environment: Option<String>,
}

#[derive(Deserialize)]
struct KvV2Response {
    data: KvV2Inner,
}
#[derive(Deserialize)]
struct KvV2Inner {
    data: ExchangeCreds,
}

/// Read `secret/data/exchanges/{exchange}` (shared / tenant-zero) from Vault.
pub async fn load_exchange_creds(
    vault_addr: &str,
    token: &str,
    exchange: &str,
) -> Result<ExchangeCreds> {
    fetch(vault_addr, token, &format!("secret/data/exchanges/{exchange}")).await
}

/// Read a tenant's broker credentials from its isolated Vault partition
/// `secret/tenants/{tenant_id}/brokers/{broker}`. The engine pulls these into
/// memory only at strategy start and drops the `ExchangeCreds` (zeroing the
/// owning allocation) on strategy stop / kill-switch — see execution::keys.
pub async fn load_tenant_broker_creds(
    vault_addr: &str,
    token: &str,
    tenant: crate::tenant::TenantId,
    broker: &str,
) -> Result<ExchangeCreds> {
    let path = format!("secret/data/tenants/{tenant}/brokers/{broker}");
    fetch(vault_addr, token, &path).await
}

async fn fetch(vault_addr: &str, token: &str, logical_path: &str) -> Result<ExchangeCreds> {
    let url = format!("{vault_addr}/v1/{logical_path}");
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("X-Vault-Token", token)
        .send()
        .await?
        .error_for_status()?
        .json::<KvV2Response>()
        .await?;
    Ok(resp.data.data)
}
