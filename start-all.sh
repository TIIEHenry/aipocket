#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"

mkdir -p /data/aipocket/results "$ROOT/logs"
cd "$ROOT"

if ss -tln | grep -q ':8000 '; then
  echo "backend already on :8000"
else
  nohup bash "$ROOT/scripts/run-backend.sh" \
    > "$ROOT/logs/backend.log" 2>&1 &
  echo $! > "$ROOT/logs/backend.pid"
  echo "backend started pid=$(cat "$ROOT/logs/backend.pid")"
fi

if ss -tln | grep -q ':3080 '; then
  echo "frontend already on :3080"
else
  cd "$ROOT/frontend"
  nohup pnpm exec vite preview --host 127.0.0.1 --port 3080 \
    > "$ROOT/logs/frontend.log" 2>&1 &
  echo $! > "$ROOT/logs/frontend.pid"
  echo "frontend started pid=$(cat "$ROOT/logs/frontend.pid")"
fi

echo "UI  http://127.0.0.1:3080"
echo "API http://127.0.0.1:8000"
