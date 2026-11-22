#!/usr/bin/env bash
# Cross-service health probe. Non-zero exit if any critical service is down.
set -uo pipefail
cd "$(dirname "$0")/.."
set -a; [ -f .env ] && . ./.env; set +a

ok=0; fail=0
check() {
  local name="$1" url="$2"
  if curl -fsS --max-time 4 "$url" >/dev/null 2>&1; then
    printf "  \033[32m✓\033[0m %-18s %s\n" "$name" "$url"; ok=$((ok+1))
  else
    printf "  \033[31m✗\033[0m %-18s %s\n" "$name" "$url"; fail=$((fail+1))
  fi
}

echo "▶ health"
check "trading-engine" "http://127.0.0.1:${ENGINE_METRICS_PORT:-8139}/metrics"
check "ai-engine"      "http://127.0.0.1:${AI_ENGINE_PORT:-8140}/health"
check "news-service"   "http://127.0.0.1:${NEWS_PORT:-8141}/metrics"
check "portfolio"      "http://127.0.0.1:${PORTFOLIO_PORT:-8142}/health"
check "dashboard"      "http://127.0.0.1:${DASHBOARD_PORT:-8130}/up"
check "prometheus"     "http://127.0.0.1:${PROMETHEUS_PORT:-8135}/-/healthy"
check "grafana"        "http://127.0.0.1:${GRAFANA_PORT:-8136}/api/health"
check "nats"           "http://127.0.0.1:${NATS_MONITOR_PORT:-8134}/healthz"

echo
echo "  $ok up, $fail down"
[ "$fail" -eq 0 ]
