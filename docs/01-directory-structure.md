# 01 目录结构规划

> 文档版本：v0.1（评审稿）
> 本文件定义项目根目录与后端 `backend/` 的完整目录树，以及每个目录/文件的职责、命名规范与分层依赖规则。
> **注意：当前仅为规划，尚未创建 `backend/` 目录与任何代码文件。**

## 1. 根目录总览

```text
/workspace
├── README.md                     # 项目入口说明（后续补充文档索引）
├── .gitignore                    # 忽略 target/、.env、临时音频、IDE 文件
├── docs/                         # 项目级设计与评审文档（本次产出）
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

说明：

1. 后端开发的一切代码、配置、迁移、测试都放在 `backend/` 内，根目录只保留文档与仓库级配置。
2. `frontend/` 仅作为未来前端占位，本期不创建、不规划技术细节。
3. 根目录 `README.md` 只做项目简介与文档索引，不写业务逻辑。

## 2. 后端完整目录树（规划）

```text
backend/
├── Cargo.toml                         # [[bin]] novelcast-server（main.rs）与 novelcast-worker（bin/worker.rs）
├── Cargo.lock                         # 依赖锁文件（提交入库）
├── rust-toolchain.toml                # 固定 Rust stable 版本，保证 CI/镜像一致
├── .env.example                       # 全部环境变量示例（无真实密钥）
├── .dockerignore                      # 构建上下文排除 target/、.git、临时音频
├── .gitignore                         # 后端局部忽略规则
├── Dockerfile                         # 多阶段构建：builder + runtime（含 ffmpeg）
├── docker-compose.yml                 # mysql + rustfs + api + worker 编排
│
├── config/                            # 非敏感默认配置（敏感项走环境变量）
│   ├── default.toml                   # 所有配置项默认值
│   ├── development.toml               # 开发环境覆盖（日志、CORS、并发）
│   └── production.toml                # 生产环境覆盖（日志 JSON、严格 CORS）
│
├── migrations/                        # sqlx migrations，按领域拆分，顺序执行
│   ├── 0001_users.sql                 # users 表 + 初始管理员由启动引导创建
│   ├── 0002_files_voices.sql          # files / voices（被 books 依赖）
│   ├── 0003_books_chapters.sql        # books / chapters
│   ├── 0004_characters_segments.sql   # characters / segments
│   ├── 0005_jobs.sql                  # jobs / job_items / audio_cache
│   └── 0006_exports.sql               # exports
│
├── assets/
│   └── voices/
│       └── edge_voices.seed.json      # Edge 音色目录种子数据（离线初始化兜底）
│
├── fixtures/                          # 测试固定数据
│   ├── novels/
│   │   ├── sample_utf8.txt            # 3 章 UTF-8 样例小说
│   │   └── sample_gbk.txt             # 3 章 GBK 样例小说（编码检测用例）
│   └── audio/
│       └── sample_segment.mp3         # 1 秒 MP3，用于 FFmpeg 拼接测试
│
├── scripts/
│   ├── dev_up.sh                      # 启动 mysql/rustfs 开发依赖
│   ├── dev_down.sh                    # 停止开发依赖（不删卷）
│   └── check.sh                       # fmt + clippy + test 一键检查
│
├── src/
│   ├── main.rs                        # API Server 入口：加载配置、初始化、启动 axum
│   ├── bin/
│   │   └── worker.rs                  # Worker 入口：加载配置、初始化、消费任务
│   ├── lib.rs                         # 公共库入口，导出各层模块
│   ├── app.rs                         # 组装 AppState、Router、中间件（仅 API 使用）
│   ├── config.rs                      # 配置结构体、加载、默认值、校验
│   ├── error.rs                       # AppError、错误码、IntoResponse、日志
│   ├── state.rs                       # AppState 定义与构造函数
│   ├── openapi.rs                     # utoipa OpenAPI 文档聚合（/docs 与 /api-docs/openapi.json）
│   │
│   ├── api/                           # HTTP 层：只做协议转换，不写 SQL、不写业务规则
│   │   ├── mod.rs
│   │   ├── router.rs                  # v1 路由表、层叠顺序、CORS/限流/追踪挂载
│   │   ├── extractors.rs              # AuthUser 提取器、分页参数、JSON 大小限制
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                # JWT 校验中间件（或提取器实现）
│   │   │   ├── request_id.rs          # 生成/透传 X-Request-Id
│   │   │   └── rate_limit.rs          # 登录接口内存限流（MVP）
│   │   ├── dto/                       # 请求/响应 DTO（serde + validator + utoipa）
│   │   │   ├── mod.rs
│   │   │   ├── common.rs              # 分页、统一响应、错误体
│   │   │   ├── auth.rs
│   │   │   ├── books.rs
│   │   │   ├── chapters.rs
│   │   │   ├── segments.rs
│   │   │   ├── characters.rs
│   │   │   ├── voices.rs
│   │   │   ├── jobs.rs
│   │   │   ├── exports.rs
│   │   │   └── files.rs
│   │   └── v1/                        # 路由处理器，按资源拆分
│   │       ├── mod.rs
│   │       ├── auth.rs                # 登录、当前用户、改密
│   │       ├── books.rs               # 书籍 CRUD、导入、封面、统计
│   │       ├── chapters.rs            # 章节列表/详情/编辑/重切/合成/拼接/音频
│   │       ├── segments.rs            # 段落列表/编辑/批量更新/单段合成/音频
│   │       ├── characters.rs          # 角色 CRUD、识别任务
│   │       ├── voices.rs              # 音色列表/同步/试听
│   │       ├── jobs.rs                # 任务列表/详情/取消/重试/SSE
│   │       ├── exports.rs             # 导出创建/查询/下载
│   │       ├── files.rs               # 文件鉴权下载
│   │       └── health.rs              # /healthz /readyz
│   │
│   ├── domain/                        # 领域层：模型、枚举、仓储接口、外部端口
│   │   ├── mod.rs
│   │   ├── models/                    # 纯数据结构，不依赖 sqlx/axum
│   │   │   ├── mod.rs
│   │   │   ├── user.rs
│   │   │   ├── book.rs
│   │   │   ├── chapter.rs
│   │   │   ├── segment.rs
│   │   │   ├── character.rs
│   │   │   ├── voice.rs
│   │   │   ├── file.rs
│   │   │   ├── job.rs
│   │   │   ├── export.rs
│   │   │   └── pagination.rs
│   │   ├── enums.rs                   # 所有状态枚举 + 字符串互转
│   │   ├── repos/                     # 仓储 trait（由 infra 实现）
│   │   │   ├── mod.rs
│   │   │   ├── user_repo.rs
│   │   │   ├── book_repo.rs
│   │   │   ├── chapter_repo.rs
│   │   │   ├── segment_repo.rs
│   │   │   ├── character_repo.rs
│   │   │   ├── voice_repo.rs
│   │   │   ├── file_repo.rs
│   │   │   ├── job_repo.rs
│   │   │   ├── export_repo.rs
│   │   │   └── audio_cache_repo.rs
│   │   └── ports/                     # 外部能力接口（由 infra 实现）
│   │       ├── mod.rs
│   │       ├── storage.rs             # ObjectStorage trait
│   │       ├── tts.rs                 # TtsProvider trait + 请求/响应类型
│   │       ├── audio.rs               # AudioProcessor trait（ffprobe/拼接/响度）
│   │       └── queue.rs               # JobQueue trait（入队、认领、心跳、完成）
│   │
│   ├── service/                       # 业务编排层：事务边界、状态流转、组合调用
│   │   ├── mod.rs
│   │   ├── auth_service.rs            # 登录、密码校验、JWT 签发
│   │   ├── book_service.rs            # 书籍 CRUD、统计、封面
│   │   ├── import_service.rs          # 文件落库、创建导入任务、重试
│   │   ├── chapter_service.rs         # 章节编辑、重切章、状态保护
│   │   ├── segment_service.rs         # 段落编辑、重切段、批量更新、失效处理
│   │   ├── speaker_service.rs         # 角色识别编排、结果落库、候选角色
│   │   ├── character_service.rs       # 角色 CRUD、音色绑定、批量推荐
│   │   ├── voice_service.rs           # 音色同步、试听、查询
│   │   ├── tts_service.rs             # 合成任务创建、缓存键、进度聚合
│   │   ├── audio_service.rs           # 拼接任务创建、章节音频失效
│   │   ├── export_service.rs          # 导出任务创建、下载、删除
│   │   └── job_service.rs             # 任务查询、取消、重试、状态机校验
│   │
│   ├── infra/                         # 基础设施层：实现 domain 的 trait
│   │   ├── mod.rs
│   │   ├── db/
│   │   │   ├── mod.rs                 # 连接池创建、健康检查
│   │   │   └── repos/                 # sqlx 实现（唯一允许写 SQL 的目录）
│   │   │       ├── mod.rs
│   │   │       ├── user_repo.rs
│   │   │       ├── book_repo.rs
│   │   │       ├── chapter_repo.rs
│   │   │       ├── segment_repo.rs
│   │   │       ├── character_repo.rs
│   │   │       ├── voice_repo.rs
│   │   │       ├── file_repo.rs
│   │   │       ├── job_repo.rs
│   │   │       ├── export_repo.rs
│   │   │       └── audio_cache_repo.rs
│   │   ├── storage/
│   │   │   ├── mod.rs
│   │   │   ├── rustfs.rs              # aws-sdk-s3，Path-Style 实现 ObjectStorage
│   │   │   └── memory.rs              # 测试用内存实现
│   │   ├── tts/
│   │   │   ├── mod.rs
│   │   │   ├── registry.rs            # Provider 注册表、默认 Provider、并发限制
│   │   │   ├── edge.rs                # Edge-TTS：音色列表 + WSS + SSML
│   │   │   ├── azure.rs               # 预留：Azure 官方 TTS（Phase 2 占位）
│   │   │   └── mock.rs                # 测试用固定 MP3 Provider
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── ffmpeg.rs              # 拼接、响度归一、编码、ID3 写入
│   │   │   └── probe.rs               # ffprobe 读取时长/采样率/声道
│   │   └── queue/
│   │       ├── mod.rs
│   │       └── mysql_queue.rs         # SKIP LOCKED 认领、心跳、回收、完成/失败
│   │
│   ├── worker/                        # 异步任务消费者（worker 二进制）
│   │   ├── mod.rs
│   │   ├── dispatcher.rs              # 主循环、并发信号量、优雅停机、心跳
│   │   ├── progress.rs                # 进度计算与批量更新
│   │   └── handlers/
│   │       ├── mod.rs
│   │       ├── import_parse.rs        # 导入 → 编码检测 → 切章 → 切段 → 角色识别
│   │       ├── character_detect.rs    # 全书角色识别（可独立重跑）
│   │       ├── chapter_tts.rs         # 批量段落合成
│   │       ├── chapter_merge.rs       # 拼接 + 响度 + 章节音频入库
│   │       └── book_export.rs         # ZIP 导出
│   │
│   └── utils/                         # 无状态工具，可独立单测
│       ├── mod.rs
│       ├── encoding.rs                # BOM/UTF-8/GB18030 检测与解码
│       ├── text.rs                    # 文本清洗、切章正则、切段、引号分类
│       ├── hash.rs                    # SHA-256 缓存键、文件校验
│       ├── id.rs                      # 请求 ID（UUID v7）、对象键生成
│       └── time.rs                    # UTC 时间、耗时格式化
│
└── tests/                             # 集成测试（每个文件独立二进制）
    ├── common/
    │   └── mod.rs                     # 测试应用、测试库、Mock Provider/Storage 装配
    ├── api_auth.rs                    # 登录/鉴权/限流
    ├── api_books.rs                   # 书籍 CRUD/分页/错误码
    ├── import_flow.rs                 # 导入 → 切章 → 切段 → 角色识别
    ├── tts_flow.rs                    # 批量合成 → 缓存命中 → 失败重试
    ├── merge_export_flow.rs           # 拼接（fixture 音频）→ 导出 ZIP
    └── job_queue.rs                   # 认领、心跳、回收、取消
```

## 3. 目录职责速查

| 目录 | 职责 | 禁止事项 |
| --- | --- | --- |
| `api/` | HTTP 协议适配、参数校验、鉴权、DTO 映射 | 写 SQL、直接调用 infra、实现业务规则 |
| `service/` | 用例编排、事务边界、状态流转、权限外的业务校验 | 处理 HTTP 类型、拼接 SQL 字符串 |
| `domain/` | 模型、枚举、仓储 trait、外部端口 trait | 依赖 sqlx/axum/aws-sdk |
| `infra/` | 实现 domain trait：MySQL、RustFS、Edge-TTS、FFmpeg、队列 | 反向依赖 service/api |
| `worker/` | 消费任务，调用 service 与 ports 完成生产流程 | 暴露 HTTP、直接返回 HTTP 错误 |
| `utils/` | 纯函数工具，无 I/O | 依赖业务状态、访问数据库 |
| `tests/` | 集成测试与流程测试 | 复用生产配置、连接生产库 |

## 4. 分层依赖规则

```text
api ──────► service ──────► domain ◄────── infra
                │              ▲
                └──────────────┘
worker ───► service / domain ports

app.rs（组装根）负责把 infra 实现注入 AppState，禁止在业务层 new 具体实现。
```

硬性规则：

1. 依赖方向只能自上而下：`api → service → domain`，`infra → domain`。
2. `service` 不出现 `axum::*`、`sqlx::*`、`aws_sdk_s3::*` 类型。
3. `domain` 不出现任何框架/驱动依赖。
4. 所有 SQL 只在 `infra/db/repos/` 内；所有 FFmpeg 参数只在 `infra/audio/` 内。
5. 事务边界统一在 `service`：一个用例一个事务，跨对象存储的操作采用「先写对象存储，再提交数据库」或补偿删除。
6. `worker` 与 `api` 共用 `lib.rs` 中的 service/domain/infra，不复制业务逻辑。

## 5. 命名规范

### 5.1 Rust

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 文件/模块 | `snake_case` | `tts_service.rs` |
| 结构体/枚举/trait | `PascalCase` | `TtsService`, `JobStatus`, `TtsProvider` |
| 函数/变量 | `snake_case` | `create_chapter_tts_job` |
| 常量 | `SCREAMING_SNAKE_CASE` | `MAX_IMPORT_BYTES` |
| 错误变体 | 语义化过去式/名词 | `BookNotFound`, `InvalidState` |

### 5.2 数据库

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 表名 | 复数 `snake_case` | `books`, `job_items` |
| 字段 | `snake_case` | `created_at`, `default_voice_id` |
| 外键 | `{单数表名}_id` | `book_id`, `chapter_id` |
| 布尔 | `is_` / `has_` 前缀，`TINYINT(1)` | `is_custom` |
| 索引 | `idx_{表}_{字段}` | `idx_jobs_status_priority` |
| 唯一键 | `uk_{表}_{字段}` | `uk_chapters_book_idx` |

### 5.3 API

| 类型 | 规范 | 示例 |
| --- | --- | --- |
| 路径 | 小写复数名词，多词用 `-` | `/api/v1/audio-settings` |
| JSON 字段 | `snake_case` | `default_narrator_voice_id` |
| 时间 | ISO-8601 UTC | `2026-11-12T08:00:00Z` |
| 资源 ID | JSON number（BIGINT 范围内） | `123` |
| 枚举值 | 小写 `snake_case` | `tts_partial` |

### 5.4 对象存储 Key

| 类型 | 规则 | 示例 |
| --- | --- | --- |
| 源文件 | `sources/{book_id}/{uuid}.{ext}` | `sources/12/0193...txt` |
| 封面 | `covers/{book_id}/{uuid}.{ext}` | `covers/12/0193...jpg` |
| 段落音频 | `audio/segments/{book_id}/{chapter_id}/{segment_id}-{cache8}.mp3` | — |
| 章节音频 | `audio/chapters/{book_id}/{chapter_id}-{rev}.mp3` | 重新拼接 rev 递增 |
| 导出 | `exports/{book_id}/{export_id}.zip` | — |
| 音色样例 | `samples/{voice_id}-{text_hash8}.mp3` | — |

## 6. 构建与运行产物

| 产物 | 说明 |
| --- | --- |
| `novelcast-server` | API 服务，默认监听 `0.0.0.0:8080`，宿主机映射 `18080` |
| `novelcast-worker` | 任务消费者，无端口；与 API 共用镜像，不同启动命令 |
| MySQL 数据卷 | `novelcast_mysql_data` |
| RustFS 数据卷 | `novelcast_rustfs_data` |
| 临时音频目录 | 容器内 `/tmp/novelcast`，任务结束清理 |

## 7. 不入库内容

```text
backend/target/
backend/.env
backend/tmp/
backend/.idea/
backend/.vscode/
*.mp3 / *.wav（fixtures 例外）
*.log
```

## 8. 评审关注点

1. 后端目录名是否采用 `backend/`（可改为 `server/`，全项目只需改根目录与 Compose 构建上下文）。
2. 是否接受「API 与 Worker 同包两个二进制」的工程结构。
3. 是否接受任务队列基于 MySQL（不引入 Redis）。
4. 是否接受领域层 `domain/ports` + `infra` 实现的端口-适配器结构（而非简单 MVC）。
