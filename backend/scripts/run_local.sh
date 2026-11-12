#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if [ ! -f .env ]; then
  cp .env.example .env
  echo "已创建 backend/.env，请按需填写 TTS_OPENAI_API_KEY / TTS_MINIMAX_API_KEY / TTS_STEPFUN_API_KEY"
fi

docker compose up -d mysql rustfs
echo "MySQL: 127.0.0.1:13306  RustFS: 127.0.0.1:19000"
echo "启动 API: http://127.0.0.1:18080/  健康检查: http://127.0.0.1:18080/healthz"
exec cargo run --bin novelcast-server
