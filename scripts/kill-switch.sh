#!/usr/bin/env bash
# Emergency kill switch. Sets the Redis flag the risk + execution services honor
# directly: risk stops approving orders; execution cancels working orders and
# market-closes open positions. Works even if the dashboard is down.
#   ./scripts/kill-switch.sh on    # flatten + halt
#   ./scripts/kill-switch.sh off   # resume
set -euo pipefail
cd "$(dirname "$0")/.."
set -a; [ -f .env ] && . ./.env; set +a

action="${1:-status}"
compose() { docker compose -f infrastructure/docker/docker-compose.yml --env-file .env "$@"; }
redis() { compose exec -T redis redis-cli -a "${REDIS_PASSWORD:-algotrader}" "$@" 2>/dev/null; }

case "$action" in
  on)
    redis SET kill_switch 1 >/dev/null
    echo "🛑 KILL SWITCH ENGAGED — new entries halted; execution will flatten."
    ;;
  off)
    redis SET kill_switch 0 >/dev/null
    echo "✓ kill switch cleared — trading may resume."
    ;;
  status)
    v="$(redis GET kill_switch || echo '?')"
    echo "kill_switch = ${v:-0}"
    ;;
  *)
    echo "usage: $0 [on|off|status]"; exit 1;;
esac
