#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
for name in backend frontend; do
  pidfile="$ROOT/logs/${name}.pid"
  if [[ -f "$pidfile" ]]; then
    pid=$(cat "$pidfile")
    if kill -0 "$pid" 2>/dev/null; then
      kill "$pid" || true
      echo "stopped $name pid=$pid"
    fi
    rm -f "$pidfile"
  fi
done
# fallback by port
fuser -k 8000/tcp 2>/dev/null || true
fuser -k 3080/tcp 2>/dev/null || true
echo done
