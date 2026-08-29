#!/usr/bin/env bash
# Local supervisor: the settings page POSTs /api/system/restart which exits 75.
# Docker Compose uses `restart: unless-stopped`; this loop covers nohup/start-all.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${AIPOCKET_BIN:-$ROOT/target/release/aipocket}"
HOST="${AIPOCKET_HOST:-127.0.0.1}"
PORT="${AIPOCKET_PORT:-8000}"
child=""

term() {
  if [[ -n "${child}" ]] && kill -0 "$child" 2>/dev/null; then
    kill "$child" 2>/dev/null || true
    wait "$child" 2>/dev/null || true
  fi
  exit 0
}
trap term TERM INT HUP

cd "$ROOT"
set -a
# shellcheck disable=SC1091
source "$ROOT/.env"
set +a

while true; do
  "$BIN" serve --host "$HOST" --port "$PORT" &
  child=$!
  set +e
  wait "$child"
  code=$?
  set -e
  child=""
  if [[ $code -ne 75 ]]; then
    exit "$code"
  fi
  sleep 0.5
done
