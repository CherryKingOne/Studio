#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -f .env ]; then
  echo "缺少 backend/.env，请先执行: cp .env.example .env" >&2
  exit 1
fi

exec cargo run --bin novelcast-worker
