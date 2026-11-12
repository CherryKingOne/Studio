# 02 系统架构与技术选型

> 文档版本：v0.2（评审稿）
> 本文定义后端的整体架构、进程模型、技术选型、部署拓扑、配置项与安全设计。
> **本版变更：代码组织改为按功能模块划分，不使用 DDD 分层。**

## 1. 架构总览

```mermaid
flowchart LR
    C[API 调用方 / 后续前端] -->|HTTP + JWT| API[novelcast-server<br/>Axum API]
    API --> F[features 功能模块<br/>books/imports/tts/audio...]
    F --> COMMON[common 共享能力<br/>storage/ffmpeg/text/db]
    API -->|创建任务| DB[(MySQL 8)]
    F -->|读写| DB
    F -->|对象读写| OBJ[(RustFS / S3)]
    W[novelcast-worker<br/>任务消费者] -->|SKIP LOCKED 认领| DB
    W --> F2[features 功能模块]
    F2 -->|读取/写入状态| DB
    F2 -->|音频上传/下载| OBJ
    W -->|WSS 合成| TTS[Edge-TTS Provider]
    W -->|拼接/响度/编码| FF[FFmpeg / ffprobe]
    W -->|进度写回| DB
```

### 1.1 进程模型

| 进程 | 二进制 | 职责 | 端口 |
| --- | --- | --- | --- |
| API 服务 | `novelcast-server` | HTTP 接口、鉴权、任务创建、SSE 进度、文件下载 | 容器 `8080`，宿主机 `18080` |
| Worker | `novelcast-worker` | 认领并执行任务：导入解析、角色识别、批量 TTS、拼接、导出 | 无 |
| MySQL | 官方镜像 | 业务数据 + 任务队列 | 宿主机 `13306` → `3306` |
| RustFS | 官方镜像 | S3 兼容对象存储 | 宿主机 `19000` → `9000`；控制台 `19001` → `9001` |

两个 Rust 进程共用同一个 Cargo 包和 `lib.rs`，通过不同 `[[bin]]` 启动，避免业务逻辑重复。

### 1.2 功能模块组织与依赖

`backend/src` 按功能划分，而不是按技术分层：

```text
src/
├── features/                 # 每个功能自成目录
│   ├── auth/                 # routes / handler / service / repo / model / dto
│   ├── books/
│   ├── imports/
│   ├── chapters/
│   ├── segments/
│   ├── characters/
│   ├── voices/
│   ├── tts/                  # provider/ 内放 Edge/Azure/Mock
│   ├── audio/
│   ├── exports/
│   ├── jobs/                 # queue.rs 实现 MySQL 任务队列
│   ├── files/
│   └── health/
├── common/                   # 与业务无关的共享能力
│   ├── db.rs / storage.rs / ffmpeg.rs
│   └── text.rs / encoding.rs / hash.rs / id.rs / pagination.rs / enums.rs
├── worker/                   # 任务入口，调用各功能 service
├── middleware/               # JWT、请求 ID、限流
├── app.rs / router.rs / state.rs / config.rs / error.rs
└── main.rs / bin/worker.rs
```

依赖规则：

1. `features/<功能>` 内部包含自己的路由、参数、业务、SQL、模型，功能内闭环。
2. 跨功能只调用对方的 `service` 公开函数，不直接调用对方的 `repo`。
3. `common` 不依赖 `features`、`worker`、`middleware`。
4. `worker/jobs/*` 只调用功能 `service`，不写 SQL。
5. SQL 只出现在各功能 `repo.rs`；FFmpeg 只在 `common/ffmpeg.rs`；S3 只在 `common/storage.rs`。
6. **不使用 `domain/`、`ports/`、`adapters/`、Repository trait 等 DDD 结构。**
7. 新增功能 = 新增 `features/<name>/` + 在 `features/mod.rs` 与 `router.rs` 注册。

## 2. 核心请求路径

### 2.1 同步请求（查询/编辑）

```text
Client → Axum Router → 中间件(request_id → auth → body limit)
      → Handler(参数校验/DTO) → 功能 service(业务校验 + 事务) → 功能 repo(MySQL)
      → DTO → 统一响应
```

### 2.2 异步任务（合成/拼接/导出/导入）

```mermaid
sequenceDiagram
    participant C as Client
    participant A as API
    participant D as MySQL
    participant W as Worker
    participant S as RustFS
    participant T as Edge-TTS
    C->>A: POST /chapters/{id}/tts
    A->>D: 校验状态 + 创建 job/job_items (事务)
    A-->>C: 202 {job_id}
    C->>A: GET /jobs/{id}/events (SSE)
    W->>D: SKIP LOCKED 认领 job → running
    loop 每个待合成段落
        W->>D: CAS pending → synthesizing
        W->>T: WSS 合成 MP3
        W->>S: 上传段落音频
        W->>D: 更新 segment + audio_cache + job 进度
        A->>D: 定时读取 job 进度
        A-->>C: SSE progress 事件
    end
    W->>D: job succeeded / partial / failed
    A-->>C: SSE done 事件
```

### 2.3 SSE 推送策略

- API 端每 `800ms` 查询一次 `jobs` 表，与上次快照比较，有变化才推送。
- 每 `15s` 发送一次注释行心跳（`: ping`），防止代理断开。
- 单个连接最长 `30min`；客户端断开后重连，重新读取当前快照。
- 该方案单机足够，不引入 Redis Pub/Sub。

## 3. 技术选型

> 版本在 `Cargo.toml` 中锁定；下表为主要候选，最终以实际可编译的稳定版本为准。

| 领域 | 选型 | 用途与理由 |
| --- | --- | --- |
| 语言 | Rust stable（Edition 2021） | 内存安全、并发友好、单二进制部署 |
| Web | `axum` 0.8 | 与 Tokio/Tower 生态一致，路由与中间件清晰 |
| 异步运行时 | `tokio` | 标准选择，支持多线程与优雅停机 |
| 中间件 | `tower-http` | CORS、Trace、RequestId、BodyLimit、压缩 |
| 数据库 | MySQL 8.0 + `sqlx` 0.8 | 连接池、运行时 SQL、JSON/时间类型 |
| 配置 | `dotenvy` + `std::env` | 全部配置从 `.env` / 环境变量读取，缺失时使用安全默认值 |
| 数据库初始化 | 代码内嵌 DDL + `schema_migrations` | 建表 SQL 全部写在 Rust 常量中，不维护 `.sql` 文件 |
| 日志 | `tracing` + `tracing-subscriber` | 结构化日志、request_id/job_id 关联 |
| 错误 | `thiserror` + 自研 `AppError` | 领域错误 → HTTP 错误码统一映射 |
| 鉴权 | `jsonwebtoken` + `argon2` | JWT Bearer；Argon2id 密码哈希 |
| 参数校验 | `validator` | DTO 层校验，错误信息可定位字段 |
| API 文档 | `utoipa` + `utoipa-swagger-ui` | 生成 OpenAPI，开发环境挂 `/docs` |
| 对象存储 | `aws-sdk-s3` | S3 兼容；对接 RustFS，强制 Path-Style |
| TTS（Edge） | `tokio-tungstenite` + `reqwest` | Edge 走 WebSocket + SSML；HTTP 引擎用 reqwest |
| 音频 | `ffmpeg` / `ffprobe` 子进程 | 稳定、参数透明；通过 `tokio::process::Command` 调用 |
| 任务队列 | MySQL 表 + `SKIP LOCKED` | 单机零额外组件；支持多 Worker 与崩溃回收 |
| ID/哈希 | `uuid`（v7）、`sha2` | 请求 ID、对象键、音频缓存键 |
| 测试 | `tokio::test` + `tower::ServiceExt` | Handler 级测试；TTS/Storage 用 Mock 实现 |

### 3.1 为什么不选 Redis

- MVP 单机、单管理员，任务吞吐远低于 MySQL 能力上限。
- MySQL 任务表天然持久化，取消/重试/审计与业务数据同库，少一个运维组件。
- 通过 `FOR UPDATE SKIP LOCKED`、心跳、超时回收即可满足可靠性。
- 后续多实例高吞吐时，可替换 `features/jobs/queue.rs` 的实现为 Redis/消息队列，业务代码不变。

### 3.2 为什么 API 与 Worker 同包

- 共享功能模块与 TTS Provider 实现，避免重复代码。
- 部署上仍是两个进程，互不影响；Worker 崩溃不会拖垮 API。
- 后续拆分独立仓库或 crate 时，只需移动 `worker` 与组装代码。

## 4. 部署拓扑（Docker Compose）

```text
┌─────────────────────────────── 单机 Docker Compose ───────────────────────────────┐
│                                                                                   │
│  api:18080 ──► mysql:3306 (volume: novelcast_mysql_data)                          │
│      │                                                                             │
│      └──────► rustfs:9000 (volume: novelcast_rustfs_data)                          │
│                                                                                   │
│  worker ─────► mysql / rustfs / 外网 Edge-TTS                                      │
│                                                                                   │
└───────────────────────────────────────────────────────────────────────────────────┘
```

| 服务 | 镜像/构建 | 端口 | 依赖 | 健康检查 |
| --- | --- | --- | --- | --- |
| `mysql` | `mysql:8.0` | `13306:3306` | — | `mysqladmin ping` |
| `rustfs` | RustFS 官方镜像 | `19000:9000`、`19001:9001` | — | TCP / HTTP 健康端点 |
| `api` | `backend/Dockerfile`，命令 `novelcast-server` | `18080:8080` | mysql healthy、rustfs healthy | `GET /healthz` |
| `worker` | 同镜像，命令 `novelcast-worker` | 无 | mysql、rustfs、api 启动后 | 进程存活（无 HTTP） |

关键约束：

1. 所有服务使用 `restart: unless-stopped`。
2. MySQL 与 RustFS 数据使用命名卷；`docker compose down` 不删除卷。
3. API 与 Worker 使用同一镜像，仅入口命令不同。
4. 时区统一 UTC（容器 `TZ=UTC`）。
5. RustFS 桶在 API 启动时确保存在（`ensure_bucket`），失败则 `/readyz` 返回 503。

## 5. 配置项清单

配置单一来源：`backend/.env`（由 `backend/.env.example` 复制）。启动时由 `dotenvy` 加载；Docker Compose 使用同一份 `.env` 注入变量，并用容器内地址覆盖 `DATABASE_URL` / `RUSTFS_ENDPOINT`。代码不再读取 TOML 配置文件。

| 环境变量 | 默认值 | 说明 |
| --- | --- | --- |
| `APP_ENV` | `development` | 运行环境（影响日志与生产校验） |
| `APP_HOST` | `0.0.0.0` | API 监听地址 |
| `APP_PORT` | `8080` | API 容器内端口 |
| `DATABASE_URL` | — | `mysql://user:pass@mysql:3306/novelcast` |
| `DB_MAX_CONNECTIONS` | `10` | 连接池上限 |
| `JWT_SECRET` | — | ≥32 字节随机串，必填 |
| `JWT_TTL_HOURS` | `24` | Token 有效期 |
| `ADMIN_USERNAME` | `admin` | 首次启动创建管理员 |
| `ADMIN_PASSWORD` | — | 首次启动创建管理员，必填；创建后不覆盖 |
| `RUSTFS_ENDPOINT` | `http://rustfs:9000` | S3 Endpoint |
| `RUSTFS_REGION` | `us-east-1` | S3 Region（RustFS 忽略但 SDK 必填） |
| `RUSTFS_ACCESS_KEY` | — | 访问密钥 |
| `RUSTFS_SECRET_KEY` | — | 密钥 |
| `RUSTFS_BUCKET` | `novelcast` | 单桶 + 前缀隔离 |
| `TTS_DEFAULT_PROVIDER` | `edge` | 默认引擎 |
| `TTS_DEFAULT_NARRATOR_VOICE_ID` | `zh-CN-YunxiNeural` | 无任何旁白音色配置时的系统默认 Edge 音色 |
| `TTS_MAX_ATTEMPTS` | `3` | 单段最大尝试次数 |
| `TTS_TIMEOUT_SECS` | `45` | 单段合成超时 |
| `TTS_CONCURRENCY` | `4` | 单任务内段落并发 |
| `TTS_PROVIDER_MIN_INTERVAL_MS` | `250` | Provider 请求最小间隔 |
| `WORKER_CONCURRENCY` | `2` | 单 Worker 进程并发任务数 |
| `WORKER_HEARTBEAT_SECS` | `10` | 任务心跳间隔 |
| `WORKER_STALE_SECS` | `90` | 心跳超时判定，超时回收 |
| `FFMPEG_PATH` | `ffmpeg` | 可执行文件路径 |
| `FFPROBE_PATH` | `ffprobe` | 可执行文件路径 |
| `FFMPEG_CONCURRENCY` | `2` | 拼接/探测并发 |
| `AUDIO_TMP_DIR` | `/tmp/novelcast` | 临时文件目录 |
| `MAX_IMPORT_BYTES` | `52428800` | 导入文件上限 50MB |
| `MAX_IMAGE_BYTES` | `5242880` | 封面上限 5MB |
| `IMPORT_MAX_CHARS` | `300` | 导入切段单段最大字符数 |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173` | 逗号分隔；生产必须显式配置 |
| `LOG_LEVEL` | `info` | `trace/debug/info/warn/error` |
| `LOG_FORMAT` | `pretty` | `pretty` / `json` |

Docker Compose 专用变量（同样来自 `backend/.env`）：

| 变量 | 说明 |
| --- | --- |
| `MYSQL_ROOT_PASSWORD` / `MYSQL_DATABASE` / `MYSQL_USER` / `MYSQL_PASSWORD` | MySQL 容器初始化与健康检查 |
| `MYSQL_PORT_HOST` | 宿主机 MySQL 端口（默认 13306） |
| `API_PORT_HOST` | 宿主机 API 端口（默认 18080） |
| `RUSTFS_PORT_HOST` / `RUSTFS_CONSOLE_PORT_HOST` | RustFS API / 控制台宿主端口 |

配置校验规则：

1. `JWT_SECRET` 长度 < 32 字节时启动失败。
2. `ADMIN_PASSWORD` 仅在首次启动（users 表为空）时必填。
3. `TTS_CONCURRENCY` 必须在 `1..=16`；`WORKER_CONCURRENCY` 必须在 `1..=8`。
4. `CORS_ALLOWED_ORIGINS` 含 `*` 时仅允许开发环境。

## 6. 数据存储与一致性

| 数据 | 位置 | 一致性策略 |
| --- | --- | --- |
| 业务元数据 | MySQL | 事务提交；状态机校验 |
| 源文件/封面/音频 | RustFS | 先上传对象 → 再写 `files`/更新业务表；失败时删除孤儿对象（尽力而为） |
| 音频缓存 | MySQL `audio_cache` + RustFS | 缓存键唯一；命中只更新 `hit_count/last_hit_at` |
| 任务进度 | MySQL `jobs` | Worker 批量/节流更新；SSE 轮询读取 |
| 临时文件 | 容器 `/tmp/novelcast/{job_id}` | finally 清理；启动时清理 24h 前残留 |

对象存储写入顺序：

1. 生成对象 Key（服务端生成，禁止客户端传入）。
2. 上传对象并校验 ETag/大小。
3. 写 `files` 表，拿到 `file_id`。
4. 在事务中更新业务记录（指向 `file_id`）。
5. 若事务失败，尽力删除刚上传对象并记录告警日志。

## 7. 安全设计

| 风险 | 措施 |
| --- | --- |
| 密码泄露 | Argon2id；日志脱敏；不在响应中返回密码哈希 |
| Token 滥用 | JWT 24h；密钥 ≥32 字节；Bearer 校验中间件统一拦截 |
| 暴力登录 | 登录接口按 IP 内存限流（默认 10 次/分钟） |
| 越权访问 | 所有资源查询带归属校验；文件下载必须经鉴权代理 |
| 上传攻击 | 扩展名 + MIME + 大小校验；文件名不落磁盘；对象 Key 服务端生成 |
| 路径穿越 | 不使用客户端文件名拼接路径；临时文件路径由 UUID 生成 |
| 命令注入 | FFmpeg 只用参数数组调用，绝不拼接 shell 字符串 |
| SSML 注入 | 文本 XML 转义后再拼 SSML；限制文本长度 |
| 对象存储暴露 | RustFS 不对外网暴露；API 代理下载；桶不设公共读 |
| CORS | 生产显式白名单；不允许 `*` |
| 请求过大 | 全局 body limit 2MB；导入/封面上传单独放宽 |

## 8. 可观测性

| 能力 | 实现 |
| --- | --- |
| 请求日志 | `tower-http Trace` + `request_id`，记录方法、路径、状态、耗时 |
| 任务日志 | 所有任务日志携带 `job_id`、`job_type`、`book_id`、`chapter_id` |
| 健康检查 | `GET /healthz` 返回进程存活；`GET /readyz` 检查 MySQL 与桶可用 |
| 错误追踪 | 统一 `AppError`，5xx 打印 `error!` 含堆栈链与 request_id |
| 审计 | `jobs` 表保留创建者、时间、尝试次数、错误信息 |
| 指标（Phase 2） | 预留 `/metrics`，统计任务耗时、TTS 成功率、缓存命中率 |

日志禁止输出：密码、JWT 完整值、RustFS Secret、完整 SSML 文本（可截断 100 字）。

## 9. 扩展性设计

| 扩展点 | 方式 |
| --- | --- |
| 新增业务功能 | 新增 `features/<name>/`，在 `features/mod.rs` 与 `router.rs` 注册 |
| 新增 TTS 引擎 | 在 `features/tts/provider/` 实现 `TtsProvider` 并注册，业务代码不改 |
| 多 Worker | 多个 worker 容器即可，`SKIP LOCKED` 自动分摊 |
| 替换队列 | 替换 `features/jobs/queue.rs` 的内部实现，service 调用不变 |
| 替换存储 | 修改 `common/storage.rs` 的后端枚举（RustFS / OSS / MinIO） |
| 本地模型 TTS | 新增 Provider，内部调用本地 HTTP 服务（GPT-SoVITS/CosyVoice） |
| 前端接入 | 按 `docs/04-api-design.md` 契约开发，无需后端改动 |

## 10. 关键技术决策记录（ADR 摘要）

| 决策 | 选择 | 备选 | 理由 |
| --- | --- | --- | --- |
| 队列 | MySQL 表 | Redis / RabbitMQ | 少组件、持久化、单机够用 |
| 代码组织 | 按功能模块（features） | 按技术分层 | 功能内闭环，新增功能成本低 |
| 进程 | API + Worker 分离 | 单进程后台任务 | 故障隔离，任务不阻塞 API |
| 存储 | RustFS | 本地磁盘 / MinIO | 用户指定；S3 兼容，可替换 |
| TTS | Edge 先行 | Azure / OpenAI | 免费跑通链路，Provider 可切换 |
| 音频 | FFmpeg 子进程 | 纯 Rust 音频库 | 参数成熟、响度/编码/元数据能力完整 |
| ID | BIGINT 自增 | UUID | 单机简单、索引友好；对外请求 ID 用 UUID v7 |
| API 风格 | REST + JSON | GraphQL | 资源模型清晰，前端接入成本低 |
