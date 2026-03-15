#!/usr/bin/env bash
set -euo pipefail

if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
  docker compose up --build "$@"
elif command -v docker-compose >/dev/null 2>&1; then
  docker-compose up --build "$@"
else
  echo "docker compose or docker-compose is required"
  exit 1
fi
