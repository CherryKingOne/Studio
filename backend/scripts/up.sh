#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -f .env ]; then
  cp .env.example .env
  echo "已创建 backend/.env，请填写 TTS API Key 后再启动。"
fi

docker compose up -d --build
PORT="$(grep -E '^API_PORT_HOST=' .env | tail -1 | cut -d= -f2)"
PORT="${PORT:-18080}"
echo "API:      http://127.0.0.1:${PORT}/"
echo "健康检查: http://127.0.0.1:${PORT}/healthz"
