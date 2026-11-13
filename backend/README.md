# NovelCast Backend

中文小说转多角色有声书的后端服务。前后端分离，本目录只包含后端。

## 1. 运行的是什么

| 进程 | 作用 | 端口 |
| --- | --- | --- |
| `novelcast-server` | **API Server**，用户请求入口 | `60181` |
| `novelcast-worker` | 异步任务消费者：导入、角色识别、TTS、拼接、导出 | 无端口 |
| MySQL 8 | 业务数据 + 任务队列 | 宿主机 `13306` |
| RustFS | S3 兼容对象存储 | 宿主机 `19000`，控制台 `19001` |

**启动后访问 `http://127.0.0.1:60181/` 即可确认 API 已运行。**

## 2. 开发者启动（源码）

项目处于 Demo 阶段，直接跑源码即可，不需要一键启动脚本。

```bash
cd backend
cp .env.example .env
# 编辑 .env，至少填写数据库/RustFS 密码；使用 TTS 时填写对应 API Key

# 终端 1：只启动依赖
docker compose up -d mysql rustfs

# 终端 2：API Server
cargo run --bin novelcast-server

# 终端 3：Worker（导入/合成/拼接/导出任务需要）
cargo run --bin novelcast-worker
```

验证：

```bash
curl http://127.0.0.1:60181/
curl http://127.0.0.1:60181/healthz
curl http://127.0.0.1:60181/readyz
```

返回示例：

```json
{
  "service": "novelcast-server",
  "version": "0.1.0",
  "status": "ok",
  "api_base": "/api/v1",
  "health": "/healthz",
  "ready": "/readyz"
}
```

> `readyz` 会检查 MySQL、RustFS、FFmpeg。任一不可用时返回 503。

## 3. 端口与环境变量

所有配置来自 `backend/.env`，模板为 `backend/.env.example`。

| 变量 | 说明 |
| --- | --- |
| `APP_HOST` / `APP_PORT` | API 监听地址与端口（默认 `0.0.0.0:60181`） |
| `API_PORT_HOST` | Docker Compose 宿主机端口（默认 `60181`） |
| `DATABASE_URL` | 本地开发用 `127.0.0.1:13306`；Compose 内覆盖为 `mysql:3306` |
| `JWT_SECRET` | JWT 密钥，至少 32 字节 |
| `JWT_TTL_HOURS` / `JWT_REFRESH_TTL_DAYS` | access_token 有效期（小时，默认 2）/ refresh_token 有效期（天，默认 30） |
| `ADMIN_USERNAME` / `ADMIN_PASSWORD` / `ADMIN_EMAIL` | 首次启动创建管理员（邮箱登录），仅在 users 表为空时生效 |
| `SMTP_HOST` / `SMTP_PORT` / `SMTP_USERNAME` / `SMTP_PASSWORD` / `SMTP_FROM` / `SMTP_TLS` | 验证码邮件发送；`SMTP_HOST` 留空进入开发模式（验证码打印到日志） |
| `RUSTFS_*` | RustFS/对象存储配置 |
| `TTS_DEFAULT_PROVIDER` | `openai` / `minimax` / `stepfun` |
| `TTS_OPENAI_*` | OpenAI TTS Key、Base URL、模型 |
| `TTS_MINIMAX_*` | MiniMax Key、GroupId、Base URL、模型 |
| `TTS_STEPFUN_*` | 阶跃星辰 Key、Base URL、模型 |
| `TTS_DEFAULT_NARRATOR_VOICE_ID` | 默认旁白音色 ID（由供应商平台提供） |
| `IMPORT_MAX_CHARS` | 导入切段单段最大字符数（默认 300） |

**不内置任何音色。** 需要先从 OpenAI / MiniMax / 阶跃星辰平台获取 `voice_id`，再登记到系统。

## 4. 第一次请求

### 4.1 登录

使用邮箱 + 密码登录（管理员邮箱由 `ADMIN_EMAIL` 配置，默认 `admin@localhost`）：

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@localhost","password":"admin123456"}'
```

响应同时返回双 Token：`data.access_token`（短期 JWT，默认 2 小时）与 `data.refresh_token`（长期令牌，默认 30 天，用于 `POST /api/v1/auth/refresh` 静默续期）。

```bash
TOKEN="把 access_token 填在这里"
```

新用户可通过邮箱验证码注册：`POST /api/v1/auth/register/code` 发送验证码 → `POST /api/v1/auth/register` 完成注册。未配置 `SMTP_HOST` 时为开发模式，验证码打印在服务端日志中（`APP_ENV=development` 时接口同时返回 `dev_code`）。完整认证接口见 [docs/api/auth](docs/api/README.md)。

### 4.2 创建书籍

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/books \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"山海拾遗","author":"某某"}'
```

### 4.3 导入 TXT

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/books/import \
  -H "Authorization: Bearer $TOKEN" \
  -F 'file=@/path/to/novel.txt' \
  -F 'title=山海拾遗' \
  -F 'author=某某'
```

返回：

```json
{ "code": 0, "message": "accepted", "data": { "book_id": 1, "job_id": 1 } }
```

Worker 必须在线，否则任务会停在 `queued`。

### 4.4 查看任务进度（SSE）

```bash
curl -N -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:60181/api/v1/jobs/1/events
```

### 4.5 登记音色

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/voices \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{
    "provider": "minimax",
    "provider_voice_id": "你的音色ID",
    "display_name": "旁白-沉稳男声",
    "locale": "zh-CN",
    "gender": "male"
  }'
```

试听：

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/voices/preview \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"voice_id":1,"text":"你好，这是一段试听。"}' \
  --output preview.mp3
```

### 4.6 合成整章

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/chapters/1/tts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"mode":"missing","force":false}'
```

### 4.7 拼接章节

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/chapters/1/merge \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"skip_failed":false}'
```

下载章节音频：

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:60181/api/v1/chapters/1/audio \
  --output chapter-1.mp3
```

### 4.8 导出整书 ZIP

```bash
curl -s -X POST http://127.0.0.1:60181/api/v1/books/1/exports \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"include_cover":true}'
```

完成后：

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://127.0.0.1:60181/api/v1/exports/1/download \
  --output book.zip
```

## 5. API 概览

| 分组 | 路径 |
| --- | --- |
| 服务信息 | `GET /`、`GET /healthz`、`GET /readyz` |
| 认证 | `POST /api/v1/auth/register/code`、`POST /auth/register`、`POST /auth/login`、`POST /auth/refresh`、`POST /auth/logout`、`GET /auth/me`、`POST /auth/password`、`POST /auth/password/forgot`、`POST /auth/password/reset` |
| 书籍 | `GET/POST /api/v1/books`、`GET/PATCH/DELETE /books/{id}`、`POST /books/import` |
| 章节 | `GET /books/{id}/chapters`、`GET/PATCH /chapters/{id}`、`POST /chapters/{id}/resegment` |
| 段落 | `GET /chapters/{id}/segments`、`PATCH /segments/{id}`、`PATCH /segments/batch` |
| 角色 | `GET/POST /books/{id}/characters`、`PATCH/DELETE /characters/{id}`、`POST /books/{id}/characters/detect` |
| 音色 | `GET/POST /voices`、`POST /voices/sync`、`POST /voices/preview`、`GET /voices/{id}/sample` |
| 合成 | `POST /chapters/{id}/tts`、`POST /segments/{id}/tts` |
| 音频 | `POST /chapters/{id}/merge`、`GET/DELETE /chapters/{id}/audio`、`GET /segments/{id}/audio` |
| 导出 | `POST /books/{id}/exports`、`GET /exports/{id}`、`GET /exports/{id}/download` |
| 任务 | `GET /jobs`、`GET /jobs/{id}`、`POST /jobs/{id}/cancel`、`POST /jobs/{id}/retry`、`GET /jobs/{id}/events` |
| 文件 | `GET /files/{id}` |

统一响应包络：

```json
{ "code": 0, "message": "ok", "data": {}, "request_id": "..." }
```

## 6. 目录结构

```text
backend/
├── src/
│   ├── main.rs              # API Server 入口
│   ├── bin/worker.rs        # Worker 入口
│   ├── router.rs            # 路由汇总
│   ├── app.rs / state.rs    # 应用组装
│   ├── features/            # 按功能划分
│   │   ├── auth/ books/ imports/ chapters/ segments/
│   │   ├── characters/ voices/
│   │   ├── tts/ audio/ exports/ jobs/ files/ health/
│   ├── common/              # 配置、DB、存储、FFmpeg、文本工具
│   └── worker/              # Dispatcher
├── migrations 已移除        # 建表 SQL 内嵌在 common/schema.rs
├── scripts/                 # check.sh（格式/Clippy/测试）
├── .env.example             # 全部环境变量模板
└── docker-compose.yml       # MySQL + RustFS + API + Worker
```

## 7. 检查与测试

```bash
./scripts/check.sh
```

等价于：

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 8. 常见问题

| 现象 | 排查 |
| --- | --- |
| `connect MySQL failed` | 确认 `docker compose up -d mysql`，或检查 `DATABASE_URL` 端口 13306 |
| `/readyz` 返回 503 | 看 `checks` 中 MySQL / RustFS / FFmpeg 哪项失败 |
| 任务一直 `queued` | Worker 没启动，执行 `cargo run --bin novelcast-worker` |
| 合成 40001 | TTS Key/GroupId 未填、音色 ID 不存在、供应商限流或余额不足 |
| 端口被占用 | 修改 `.env` 中 `API_PORT_HOST` / `MYSQL_PORT_HOST` / `RUSTFS_PORT_HOST` |
| 没有内置音色 | 正常设计，先 `POST /api/v1/voices` 手动登记 |
