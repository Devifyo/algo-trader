#!/usr/bin/env bash
# Logical backup of the database (pg_dump). For production add WAL archiving for
# point-in-time recovery and ship dumps off-box. Test a restore monthly.
set -euo pipefail
cd "$(dirname "$0")/.."
set -a; [ -f .env ] && . ./.env; set +a

ts="$(date -u +%Y%m%dT%H%M%SZ)"
out="backups"; mkdir -p "$out"
file="$out/algotrader_${ts}.sql.gz"

echo "▶ backing up to $file"
docker compose -f infrastructure/docker/docker-compose.yml --env-file .env \
  exec -T postgres pg_dump -U "${POSTGRES_USER:-algotrader}" "${POSTGRES_DB:-algotrader}" \
  | gzip > "$file"

echo "✓ done ($(du -h "$file" | cut -f1))"
# Retain last 14 dumps.
ls -1t "$out"/algotrader_*.sql.gz | tail -n +15 | xargs -r rm -f
