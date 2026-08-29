#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
mkdir -p /data/aipocket/results
# load .env for the process
exec "$ROOT/scripts/run-backend.sh"
