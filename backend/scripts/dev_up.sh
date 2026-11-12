#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
docker compose up -d mysql rustfs
echo "MySQL: 127.0.0.1:13306  RustFS: 127.0.0.1:19000 (console 19001)"
