#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="$ROOT_DIR/.env"

if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

PINGORA_LISTEN="${PINGORA_LISTEN:-127.0.0.1:8088}"
PINGORA_WEB_UPSTREAM="${PINGORA_WEB_UPSTREAM:-127.0.0.1:8000}"
PINGORA_SOCKETIO_UPSTREAM="${PINGORA_SOCKETIO_UPSTREAM:-127.0.0.1:9000}"
PINGORA_SITE_HOST="${PINGORA_SITE_HOST:-erpnext.localhost}"
REQUESTS="${PINGORA_GATE_REQUESTS:-500}"
CONCURRENCY="${PINGORA_GATE_CONCURRENCY:-20}"

base_url="http://$PINGORA_LISTEN"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Missing command: $1" >&2
    exit 1
  }
}

check() {
  local label="$1"
  shift
  if "$@"; then
    echo "OK   $label"
  else
    echo "FAIL $label" >&2
    exit 1
  fi
}

require_cmd cargo
require_cmd curl

cd "$ROOT_DIR"

check "cargo test" cargo test --quiet
check "release build" cargo build --release --quiet

if ! ss -ltn | grep -q ":${PINGORA_LISTEN##*:} "; then
  PINGORA_LISTEN="$PINGORA_LISTEN" \
  PINGORA_WEB_UPSTREAM="$PINGORA_WEB_UPSTREAM" \
  PINGORA_SOCKETIO_UPSTREAM="$PINGORA_SOCKETIO_UPSTREAM" \
  PINGORA_SITE_HOST="$PINGORA_SITE_HOST" \
    "$ROOT_DIR/scripts/start-prod.sh"
  started_here=1
else
  started_here=0
fi

cleanup() {
  if [ "$started_here" = "1" ]; then
    "$ROOT_DIR/scripts/stop-prod.sh" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

check "health endpoint" bash -lc "curl -fsS '$base_url/_pingora_health' | grep -qx ok"
check "ERPNext web route" bash -lc "curl -fsSI '$base_url/' | grep -q '^HTTP/1.1 200'"
check "server header" bash -lc "curl -fsSI '$base_url/' | grep -qi '^server: pingora-erpnext'"
check "socket.io polling route" bash -lc "curl -fsS '$base_url/socket.io/?EIO=4&transport=polling' | grep -q 'pingInterval'"

if command -v ab >/dev/null 2>&1; then
  ab_output="$(mktemp)"
  ab -n "$REQUESTS" -c "$CONCURRENCY" "$base_url/_pingora_health" > "$ab_output"
  failed="$(awk -F: '/Failed requests/ {gsub(/ /, "", $2); print $2}' "$ab_output")"
  non2xx="$(awk -F: '/Non-2xx responses/ {gsub(/ /, "", $2); print $2}' "$ab_output")"
  failed="${failed:-0}"
  non2xx="${non2xx:-0}"
  if [ "$failed" != "0" ] || [ "$non2xx" != "0" ]; then
    cat "$ab_output"
    echo "FAIL load test failed_requests=$failed non_2xx=$non2xx" >&2
    exit 1
  fi
  rps="$(awk '/Requests per second/ {print $4}' "$ab_output")"
  echo "OK   load test ${REQUESTS} requests concurrency=${CONCURRENCY} rps=${rps:-unknown}"
  rm -f "$ab_output"
else
  echo "WARN ab not found; skipped load test"
fi

echo "Pingora ERPNext production gate passed."
