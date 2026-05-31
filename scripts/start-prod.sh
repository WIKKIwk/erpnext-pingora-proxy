#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT_DIR/.env"
BIN="$ROOT_DIR/target/release/pingora-proxy"
PID_FILE="$ROOT_DIR/logs/pingora-prod.pid"
LOG_FILE="$ROOT_DIR/logs/pingora-prod.log"

if [ -f "$ENV_FILE" ]; then
  set -a
  source "$ENV_FILE"
  set +a
fi

PINGORA_LISTEN="${PINGORA_LISTEN:-127.0.0.1:8088}"

cd "$ROOT_DIR"
mkdir -p logs

if ss -ltn | grep -q ":${PINGORA_LISTEN##*:} "; then
  echo "Port ${PINGORA_LISTEN##*:} is already in use. Stop the old proxy first."
  exit 1
fi

cargo build --release

setsid "$BIN" > "$LOG_FILE" 2>&1 < /dev/null &
echo "$!" > "$PID_FILE"

echo "Pingora ERPNext proxy started: http://$PINGORA_LISTEN"
echo "Log: $LOG_FILE"
