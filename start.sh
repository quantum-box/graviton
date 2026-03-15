#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
trap 'jobs -p | xargs -r kill' EXIT INT TERM

cd "$ROOT_DIR/frontend"
npm install

cd "$ROOT_DIR/backend"
cargo build
cargo run &
BACKEND_PID=$!

cd "$ROOT_DIR/frontend"
if [[ "${1:-dev}" == "build" ]]; then
  npm run build
  npm run start &
else
  npm run dev &
fi
FRONTEND_PID=$!

wait "$BACKEND_PID" "$FRONTEND_PID"
