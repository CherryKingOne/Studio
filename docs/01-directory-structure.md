# 01 目录结构规划（按功能模块组织）

> 文档版本：v0.2（评审稿）
> **本版变更：废弃 DDD 分层（不再有 `domain/`、`ports/`、`adapters/`、仓储 trait），改为按功能模块组织。**
> 本文件定义根目录与后端 `backend/` 的完整目录树、每个文件职责、功能边界与命名规范。
> **状态：已进入开发，M0 工程骨架与 M1 基础接口已落地；本文件作为后续功能的目标结构基准。**

## 1. 组织原则

### 1.1 按功能划分，而不是按技术分层

每个业务功能是一个独立目录，目录内自带该功能的全部代码：

```text
features/<功能名>/
├── routes.rs      # 该功能负责的 HTTP 路由
├── handler.rs     # 参数提取、校验、调用 service、返回响应
├── service.rs     # 业务规则、状态流转、事务编排
├── repo.rs        # 该功能相关表的 SQL 访问
├── model.rs       # 数据库行结构体
├── dto.rs         # 请求/响应 DTO
└── <专用文件>      # 如 parser.rs / detector.rs / cache.rs / queue.rs / provider/
```

原则：

1. **一个功能一个目录**：新增功能 = 新增 `features/<name>/`，并在 `features/mod.rs` 与 `router.rs` 注册。
2. **功能内闭环**：路由、参数、业务、SQL、模型都放在功能目录内，不跨目录「按层」拆分。
3. **共享能力下沉到 `common/`**：对象存储、FFmpeg、文本工具、编码检测、哈希、分页、通用枚举。
4. **功能之间只通过 `service` 公开函数调用**，不直接调用对方的 `repo` 或复制对方的 SQL。
5. **不引入 DDD 概念**：没有领域层、端口、适配器、聚合根、仓储 trait；只有功能模块、服务函数、数据访问函数。
6. **Worker 复用功能模块**：异步任务 Handler 只做「取任务 → 调功能 service → 落结果」，不复制业务逻辑。

### 1.2 为什么不用 DDD 分层

| 问题 | 说明 |
| --- | --- |
| 目录跳转成本 | 改一个接口要在 api/domain/service/infra 四处找文件 |
| 过度抽象 | 单机单管理员项目不需要为每个表定义 Repository trait |
| 功能边界不清 | 按层组织时，一个功能被拆散；按功能组织时，代码集中、易删易改 |
| 与团队习惯不符 | 按功能添加代码更符合「加一个功能就加一个目录」的直觉 |

## 2. 根目录总览

```text
/workspace
├── README.md                     # 项目入口与文档索引
├── .gitignore                    # 忽略 target/、.env、临时音频、IDE 文件
├── docs/                         # 项目级设计与评审文档
│   ├── 00-overview.md
│   ├── 01-directory-structure.md
│   ├── 02-architecture.md
│   ├── 03-database-design.md
│   ├── 04-api-design.md
│   ├── 05-core-workflows.md
│   ├── 06-backend-modules.md
│   ├── 07-roadmap-testing.md
│   └── 08-review-checklist.md
├── backend/                      # 后端根目录（英文名，审核通过后创建）
└── frontend/                     # 前端预留目录（本期不创建，技术栈未定）
```

## 3. 后端完整目录树（按功能组织）

```text
backend/
├── Cargo.toml                         # [[bin]] novelcast-server（main.rs）与 novelcast-worker（bin/worker.rs）
├── Cargo.lock                         # 依赖锁文件（提交入库）
├── rust-toolchain.toml                # 固定 Rust stable 版本
├── .env.example                       # 环境变量示例（无真实密钥）
├── .dockerignore
├── .gitignore
├── Dockerfile                         # 多阶段构建，运行镜像含 ffmpeg
├── docker-compose.yml                 # mysql + rustfs + api + worker
│
├── config/                            # 非敏感默认配置
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
│
├── migrations/                        # sqlx migrations
│   ├── 0001_users.sql
│   ├── 0002_files_voices.sql
│   ├── 0003_books_chapters.sql
│   ├── 0004_characters_segments.sql
│   ├── 0005_jobs.sql                  # jobs / job_items / audio_cache
│   └── 0006_exports.sql
│
├── assets/
│   └── voices/
│       └── edge_voices.seed.json      # Edge 音色种子数据
│
├── fixtures/                          # 测试固定数据
│   ├── novels/
│   │   ├── sample_utf8.txt
│   │   └── sample_gbk.txt
│   └── audio/
│       └── sample_segment.mp3
│
├── scripts/
│   ├── dev_up.sh
│   ├── dev_down.sh
│   └── check.sh                       # fmt + clippy + test
│
├── src/
│   ├── main.rs                        # API 入口：加载配置、初始化、启动 axum
│   ├── bin/
│   │   └── worker.rs                  # Worker 入口：初始化、启动 Dispatcher
│   ├── lib.rs                         # 库入口，导出 modules
│   ├── app.rs                         # 构建 AppState、合并路由、挂中间件
│   ├── router.rs                      # 汇总所有功能路由
│   ├── config.rs                      # 配置结构与校验
│   ├── error.rs                       # AppError、错误码、统一响应
│   ├── state.rs                       # AppState / WorkerState
│   ├── openapi.rs                     # OpenAPI 聚合
│   │
│   ├── middleware/                    # 全局 HTTP 中间件（与具体功能无关）
│   │   ├── mod.rs
│   │   ├── auth.rs                    # JWT 校验提取器
│   │   ├── request_id.rs              # X-Request-Id
│   │   └── rate_limit.rs              # 登录/试听限流
│   │
│   ├── features/                      # 按功能组织的业务代码
│   │   ├── mod.rs                     # 声明所有功能模块
│   │   │
│   │   ├── health/                    # 健康检查
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs
│   │   │   └── handler.rs
│   │   │
│   │   ├── auth/                      # 登录、当前用户、改密
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── repo.rs                # users 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── books/                     # 书籍 CRUD、封面、统计、书籍设置
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── repo.rs                # books 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── imports/                   # TXT 导入与解析流水线
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # POST /books/import
│   │   │   ├── handler.rs
│   │   │   ├── service.rs             # 导入编排、重试、状态更新
│   │   │   ├── parser.rs              # 导入流水线编排（调用 common 的编码/切章/切段）
│   │   │   └── dto.rs
│   │   │
│   │   ├── chapters/                  # 章节 CRUD、重新切段入口、章节统计
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /books/{id}/chapters、/chapters/{id}、/resegment
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── repo.rs                # chapters 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── segments/                  # 段落 CRUD、批量编辑
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /chapters/{id}/segments、/segments/*
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── repo.rs                # segments 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── characters/                # 角色 CRUD、角色识别
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /books/{id}/characters、/characters/*
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── detector.rs            # 归属句式、候选池、置信度
│   │   │   ├── repo.rs                # characters 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── voices/                    # 音色目录、同步、试听
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /voices/*
│   │   │   ├── handler.rs
│   │   │   ├── service.rs
│   │   │   ├── repo.rs                # voices 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── tts/                       # 批量/单段合成任务
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /chapters/{id}/tts、/segments/{id}/tts
│   │   │   ├── handler.rs
│   │   │   ├── service.rs             # 任务创建、缓存键、进度、重试
│   │   │   ├── cache.rs               # audio_cache 读写
│   │   │   ├── repo.rs                # audio_cache 表
│   │   │   ├── dto.rs
│   │   │   └── provider/              # TTS 多引擎实现（功能内抽象）
│   │   │       ├── mod.rs             # TtsProvider trait + Registry
│   │   │       ├── edge.rs            # Edge-TTS：音色列表 + WSS + SSML
│   │   │       ├── azure.rs           # 预留
│   │   │       └── mock.rs            # 测试用固定 MP3
│   │   │
│   │   ├── audio/                     # 章节拼接、章节/段落音频下载与失效
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /chapters/{id}/merge、/chapters/{id}/audio、/segments/{id}/audio
│   │   │   ├── handler.rs
│   │   │   ├── service.rs             # 拼接任务、失效逻辑、参数合并
│   │   │   └── dto.rs
│   │   │
│   │   ├── exports/                   # 整书 ZIP 导出
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /books/{id}/exports、/exports/*
│   │   │   ├── handler.rs
│   │   │   ├── service.rs             # ZIP 组装、manifest、命名清洗
│   │   │   ├── repo.rs                # exports 表
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   ├── jobs/                      # 任务列表、取消、重试、SSE、任务队列
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs              # /jobs/*
│   │   │   ├── handler.rs
│   │   │   ├── service.rs             # 状态机、取消、重试、进度计算
│   │   │   ├── queue.rs               # MySQL 队列入队/认领/心跳/回收
│   │   │   ├── repo.rs                # jobs / job_items
│   │   │   ├── model.rs
│   │   │   └── dto.rs
│   │   │
│   │   └── files/                     # 文件下载（封面/音频/ZIP）
│   │       ├── mod.rs
│   │       ├── routes.rs              # /files/{id}
│   │       ├── handler.rs
│   │       ├── service.rs
│   │       └── repo.rs                # files 表
│   │
│   ├── worker/                        # 异步任务入口（只做编排，不写业务规则）
│   │   ├── mod.rs
│   │   ├── dispatcher.rs              # 主循环、并发、心跳、优雅停机
│   │   ├── progress.rs                # 进度节流上报
│   │   └── jobs/
│   │       ├── mod.rs
│   │       ├── import_parse.rs        # 调 imports::service::run_import
│   │       ├── character_detect.rs    # 调 characters::service::run_detect
│   │       ├── chapter_tts.rs         # 调 tts::service::run_tts
│   │       ├── chapter_merge.rs       # 调 audio::service::run_merge
│   │       └── book_export.rs         # 调 exports::service::run_export
│   │
│   └── common/                        # 与业务无关的共享能力
│       ├── mod.rs
│       ├── db.rs                      # 连接池、迁移、健康检查
│       ├── storage.rs                 # RustFS（S3）读写；测试内存后端
│       ├── ffmpeg.rs                  # ffprobe/拼接/响度/静音生成
│       ├── text.rs                    # 文本清洗、切章正则、切段、引号分类
│       ├── encoding.rs                # UTF-8/BOM/GB18030 检测
│       ├── hash.rs                    # SHA-256、缓存键
│       ├── id.rs                      # 请求 ID、对象 Key、worker_id
│       ├── pagination.rs              # 分页参数与 Page<T>
│       ├── enums.rs                   # 全局状态枚举（BookStatus/JobStatus 等）
│       └── time.rs                    # UTC 时间工具
│
└── tests/                             # 集成测试
    ├── common/
    │   └── mod.rs                     # 测试 App、测试库、Mock TTS 装配
    ├── api_auth.rs
    ├── api_books.rs
    ├── import_flow.rs
    ├── tts_flow.rs
    ├── merge_export_flow.rs
    └── job_queue.rs
```

## 4. 功能模块职责表

| 功能目录 | 负责的 API 路径 | 主要数据表 | 说明 |
| --- | --- | --- | --- |
| `health` | `/healthz`、`/readyz` | — | 存活与依赖检查 |
| `auth` | `/api/v1/auth/*` | `users` | 登录、当前用户、改密 |
| `books` | `/api/v1/books`、`/books/{id}`、`/books/{id}/cover`、`/books/{id}/stats` | `books` | 书籍 CRUD、封面、统计、书籍设置 |
| `imports` | `/api/v1/books/import` | `books`、`chapters`、`segments`、`jobs` | TXT 导入与解析流水线 |
| `chapters` | `/api/v1/books/{id}/chapters`、`/chapters/{id}`、`/chapters/{id}/resegment` | `chapters` | 章节查询/编辑/重切 |
| `segments` | `/api/v1/chapters/{id}/segments`、`/segments/*` | `segments` | 段落查询/编辑/批量更新 |
| `characters` | `/api/v1/books/{id}/characters`、`/characters/*`、`/books/{id}/characters/detect` | `characters`、`segments` | 角色 CRUD、识别 |
| `voices` | `/api/v1/voices/*` | `voices`、`files` | 音色同步、试听 |
| `tts` | `/api/v1/chapters/{id}/tts`、`/segments/{id}/tts` | `segments`、`audio_cache`、`files`、`jobs` | 合成任务与缓存 |
| `audio` | `/api/v1/chapters/{id}/merge`、`/chapters/{id}/audio`、`/segments/{id}/audio` | `chapters`、`segments`、`files`、`jobs` | 拼接、音频下载、失效 |
| `exports` | `/api/v1/books/{id}/exports`、`/exports/*` | `exports`、`files`、`jobs` | ZIP 导出 |
| `jobs` | `/api/v1/jobs/*` | `jobs`、`job_items` | 任务查询、取消、重试、SSE、队列 |
| `files` | `/api/v1/files/{id}` | `files` | 鉴权下载 |

路由归属规则：

1. 路径按「资源所属功能」划分，例如章节音频由 `audio` 功能注册，段落 CRUD 由 `segments` 功能注册。
2. 各功能 `routes.rs` 返回 `Router<Arc<AppState>>`，由 `router.rs` 统一 `merge`。
3. 同一个路径前缀可以被多个功能注册不同子路径，互不冲突。

## 5. 共享目录 `common/` 的边界

| 文件 | 放什么 | 不放什么 |
| --- | --- | --- |
| `db.rs` | 连接池、迁移、健康检查 | 具体表的 SQL（放各功能 `repo.rs`） |
| `storage.rs` | RustFS 上传/下载/删除、内存测试后端 | 业务对象 Key 规则（放 `id.rs` 或功能内） |
| `ffmpeg.rs` | ffprobe、拼接、响度、静音、ID3 | 任务状态、数据库读写 |
| `text.rs` | 纯文本算法（切章/切段/分类） | 数据库、文件 I/O |
| `encoding.rs` | 编码检测与解码 | 导入流程编排 |
| `hash.rs` | SHA-256、缓存键拼接 | 业务参数含义 |
| `id.rs` | UUID v7、对象 Key、worker_id | 业务逻辑 |
| `pagination.rs` | `Page<T>`、分页参数 | 具体查询 |
| `enums.rs` | 跨功能共享的状态枚举 | 表结构体（放各功能 `model.rs`） |

## 6. 依赖规则（功能之间）

```text
main.rs / app.rs / router.rs
        │
        ▼
   features/*  ──────► common/*
        │
        └──► features/其他功能（仅调用其 service 公开函数）

worker/dispatcher ──► features/* ──► common/*
middleware ─────────► common/*（JWT 等）
```

硬性规则：

1. `common/` 不依赖 `features/`、`worker/`、`middleware/`。
2. 功能之间允许调用对方的 `service` 公开函数；**禁止直接调用对方的 `repo`**。
3. 跨功能的写事务由发起流程的功能开启，其他功能函数接收 `&mut Transaction` 参与。
4. `worker/jobs/*` 只调用功能 `service`，不写 SQL、不做业务判断。
5. `middleware/` 只依赖 `common/` 与 `state.rs`，不依赖具体功能。
6. SQL 只出现在各功能的 `repo.rs`；FFmpeg 参数只在 `common/ffmpeg.rs`；S3 调用只在 `common/storage.rs`。
7. 不允许出现 `domain/`、`ports/`、`adapters/`、`repositories/` 等 DDD 目录。

## 7. 命名规范

### 7.1 Rust

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 文件/模块 | `snake_case` | `routes.rs`、`detector.rs` |
| 功能目录 | 复数小写 | `books/`、`segments/`、`exports/` |
| 结构体/枚举/trait | `PascalCase` | `BookService`、`JobStatus` |
| 函数/变量 | `snake_case` | `create_import_job` |
| 常量 | `SCREAMING_SNAKE_CASE` | `MAX_IMPORT_BYTES` |

### 7.2 数据库

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 表名 | 复数 `snake_case` | `books`、`job_items` |
| 字段 | `snake_case` | `created_at`、`default_voice_id` |
| 外键 | `{单数表名}_id` | `book_id`、`chapter_id` |
| 索引 | `idx_{表}_{字段}` | `idx_jobs_status_priority` |
| 唯一键 | `uk_{表}_{字段}` | `uk_chapters_book_idx` |

### 7.3 API

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 路径 | 小写复数名词，多词用 `-` | `/api/v1/audio-settings` |
| JSON 字段 | `snake_case` | `default_narrator_voice_id` |
| 时间 | ISO-8601 UTC | `2026-11-12T08:00:00Z` |
| 枚举值 | 小写 `snake_case` | `tts_partial` |

### 7.4 对象存储 Key

| 类型 | 规则 | 示例 |
| --- | --- | --- |
| 源文件 | `sources/{book_id}/{uuid}.{ext}` | `sources/12/0193...txt` |
| 封面 | `covers/{book_id}/{uuid}.{ext}` | `covers/12/0193...jpg` |
| 段落音频 | `audio/segments/{book_id}/{chapter_id}/{segment_id}-{cache8}.mp3` | — |
| 章节音频 | `audio/chapters/{book_id}/{chapter_id}-{rev}.mp3` | — |
| 导出 | `exports/{book_id}/{export_id}.zip` | — |
| 音色样例 | `samples/{voice_id}-{text_hash8}.mp3` | — |

## 8. 构建与运行产物

| 产物 | 说明 |
| --- | --- |
| `novelcast-server` | API 服务，容器内 `8080`，宿主机 `18080` |
| `novelcast-worker` | 任务消费者，无端口 |
| MySQL 数据卷 | `novelcast_mysql_data` |
| RustFS 数据卷 | `novelcast_rustfs_data` |
| 临时音频目录 | 容器内 `/tmp/novelcast` |

## 9. 不入库内容

```text
backend/target/
backend/.env
backend/tmp/
backend/.idea/
backend/.vscode/
*.mp3 / *.wav（fixtures 例外）
*.log
```

## 10. 与 v0.1 的差异

| 项 | v0.1（DDD 分层） | v0.2（按功能） |
| --- | --- | --- |
| 组织方式 | api / domain / service / infra | features/<功能>/ |
| 数据访问 | 每个表一个 Repository trait | 每个功能一个 `repo.rs` 函数集合 |
| 外部能力 | `domain/ports` + infra 实现 | `common/storage.rs`、`common/ffmpeg.rs`、`features/jobs/queue.rs`、`features/tts/provider/` |
| 新增功能 | 要改 4 个层目录 | 只加一个 `features/<name>/` |
| Worker | 复用 service/domain | 复用各功能 service |

## 11. 评审关注点

1. 是否接受「一个功能一个目录、目录内自带 routes/handler/service/repo/model/dto」。
2. 是否接受跨功能只通过 `service` 调用（不共享 repo）。
3. 是否接受 `common/` 只放与业务无关的共享能力。
4. 是否接受 TTS 多引擎抽象放在 `features/tts/provider/` 内（而不是独立端口层）。
5. 后端目录名仍为 `backend/`。
