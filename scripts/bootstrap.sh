#!/usr/bin/env bash
# One-time setup: .env, app key, Vault seed hint, build images.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "▶ algo-trader bootstrap"

if [ ! -f .env ]; then
  cp .env.example .env
  echo "  • created .env from .env.example — review secrets before going further"
fi

# Generate a Laravel APP_KEY if missing (base64:32 bytes).
if ! grep -q '^APP_KEY=base64:' .env; then
  KEY="base64:$(head -c 32 /dev/urandom | base64)"
  if grep -q '^APP_KEY=' .env; then
    sed -i "s|^APP_KEY=.*|APP_KEY=${KEY}|" .env
  else
    echo "APP_KEY=${KEY}" >> .env
  fi
  echo "  • generated APP_KEY"
fi

echo "  • building images (this compiles the Rust engine; first run is slow)…"
docker compose -f infrastructure/docker/docker-compose.yml --env-file .env build

cat <<'EOF'

✓ bootstrap complete.

Next:
  make up-infra     # data plane (postgres, redis, nats, vault)
  ./scripts/vault-seed.sh   # (optional) seed exchange testnet keys into Vault
  make up-core      # engine + ai + news + portfolio
  make up-app       # dashboard + monitoring
  make health

Dashboard  http://127.0.0.1:8130
Grafana    http://127.0.0.1:8136
EOF
