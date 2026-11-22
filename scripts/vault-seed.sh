#!/usr/bin/env bash
# Seed exchange / tenant broker credentials into Vault (dev mode). In production
# use sealed Vault + AppRole; never commit real keys. This is a convenience for
# wiring testnet keys so the engine can run the exchange market-data source.
#
#   ./scripts/vault-seed.sh binance <api_key> <api_secret>
#   ./scripts/vault-seed.sh tenant <tenant_uuid> zerodha <api_key> <api_secret>
set -euo pipefail
cd "$(dirname "$0")/.."
set -a; [ -f .env ] && . ./.env; set +a

compose() { docker compose -f infrastructure/docker/docker-compose.yml --env-file .env "$@"; }
vault() { compose exec -T -e VAULT_TOKEN="${VAULT_DEV_ROOT_TOKEN:-dev-only-root-token}" \
            -e VAULT_ADDR="http://127.0.0.1:8200" vault vault "$@"; }

mode="${1:-}"
case "$mode" in
  binance|bybit|okx|coinbase)
    vault kv put "secret/exchanges/$1" api_key="${2:?key}" api_secret="${3:?secret}"
    echo "✓ seeded secret/exchanges/$1"
    ;;
  tenant)
    tid="${2:?tenant uuid}"; broker="${3:?broker}"
    vault kv put "secret/tenants/${tid}/brokers/${broker}" api_key="${4:?key}" api_secret="${5:?secret}"
    echo "✓ seeded secret/tenants/${tid}/brokers/${broker}"
    ;;
  *)
    echo "usage:"
    echo "  $0 binance|bybit|okx|coinbase <api_key> <api_secret>"
    echo "  $0 tenant <tenant_uuid> <broker> <api_key> <api_secret>"
    exit 1;;
esac
