# 06 后端模块详细设计

> 文档版本：v0.1（评审稿）
> 本文按模块说明「职责、关键文件、关键类型、方法步骤、错误处理、验收点」，与 [01 目录结构](./01-directory-structure.md)、[03 数据库](./03-database-design.md)、[04 API](./04-api-design.md)、[05 流程](./05-core-workflows.md) 一一对应。

## 1. 代码组织与依赖注入

### 1.1 入口与组装

```text
main.rs（API）                      bin/worker.rs（Worker）
    │                                    │
    ├─ 加载配置 config.rs                ├─ 加载配置
    ├─ 初始化 tracing                    ├─ 初始化 tracing
    ├─ 连接 MySQL + migrate              ├─ 连接 MySQL
    ├─ bootstrap（管理员/桶/音色）        ├─ 构建 WorkerState
    ├─ 构建 AppState（注入实现）          └─ Dispatcher::run()
    └─ axum::serve
```

### 1.2 `AppState` 设计

```rust
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: MySqlPool,
    pub repos: Arc<Repositories>,     // 各仓储 trait 对象集合
    pub storage: Arc<dyn ObjectStorage>,
    pub tts: Arc<TtsRegistry>,
    pub audio: Arc<dyn AudioProcessor>,
    pub queue: Arc<dyn JobQueue>,
}
```

`Repositories` 聚合仓储 trait 对象：

```rust
pub struct Repositories {
    pub users: Arc<dyn UserRepo>,
    pub books: Arc<dyn BookRepo>,
    pub chapters: Arc<dyn ChapterRepo>,
    pub segments: Arc<dyn SegmentRepo>,
    pub characters: Arc<dyn CharacterRepo>,
    pub voices: Arc<dyn VoiceRepo>,
    pub files: Arc<dyn FileRepo>,
    pub jobs: Arc<dyn JobRepo>,
    pub exports: Arc<dyn ExportRepo>,
    pub audio_cache: Arc<dyn AudioCacheRepo>,
}
```

组装规则：

1. 具体实现只在 `main.rs` / `worker.rs` 中创建，然后注入。
2. Service 构造函数接收 `Arc<...>` 依赖，不自行创建数据库连接或 Provider。
3. 测试使用 `infra/storage/memory.rs`、`infra/tts/mock.rs` 与测试 MySQL 库替换实现。

### 1.3 `WorkerState`

```rust
pub struct WorkerState {
    pub config: Arc<AppConfig>,
    pub repos: Arc<Repositories>,
    pub storage: Arc<dyn ObjectStorage>,
    pub tts: Arc<TtsRegistry>,
    pub audio: Arc<dyn AudioProcessor>,
    pub queue: Arc<dyn JobQueue>,
    pub worker_id: String,
}
```

## 2. 配置模块 `config.rs`

### 2.1 职责

- 加载 `config/default.toml`、`config/{APP_ENV}.toml` 与环境变量。
- 反序列化为强类型 `AppConfig`，启动时校验非法值。
- 提供 `Arc<AppConfig>` 给 API 与 Worker。

### 2.2 关键结构（节选）

```rust
pub struct AppConfig {
    pub app: AppSection,
    pub database: DatabaseSection,
    pub jwt: JwtSection,
    pub rustfs: RustFsSection,
    pub tts: TtsSection,
    pub worker: WorkerSection,
    pub audio: AudioSection,
    pub limits: LimitsSection,
    pub cors: CorsSection,
}

pub struct TtsSection {
    pub default_provider: String,
    pub default_narrator_voice_id: String, // 如 zh-CN-YunxiNeural
    pub max_attempts: u8,
    pub timeout_secs: u64,
    pub concurrency: usize,
    pub provider_min_interval_ms: u64,
}
```

### 2.3 加载步骤

1. 读取 `APP_ENV`（默认 `development`）。
2. 使用 `figment` 合并 `default.toml` → `{env}.toml` → 环境变量（`NOVELCAST__` 前缀）。
3. 反序列化 `AppConfig`。
4. 校验：
   - `jwt.secret` ≥32 字节；
   - 并发数范围；
   - 路径非空；
   - 生产环境 CORS 不允许 `*`。
5. 返回配置；失败直接退出并打印可定位的错误。

## 3. 错误模块 `error.rs`

### 3.1 错误类型

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("unauthorized")] Unauthorized,
    #[error("forbidden")] Forbidden,
    #[error("{resource} not found")] NotFound { resource: &'static str },
    #[error("validation failed")] Validation(Vec<FieldError>),
    #[error("invalid state: {0}")] InvalidState(String),
    #[error("duplicate: {0}")] Duplicate(String),
    #[error("resource in use: {0}")] ResourceInUse(String),
    #[error("unsupported file type")] UnsupportedFileType,
    #[error("file too large")] FileTooLarge,
    #[error("storage error: {0}")] Storage(String),
    #[error("tts provider error: {0}")] TtsProvider(String),
    #[error("tts rate limited")] TtsRateLimited,
    #[error("audio process error: {0}")] AudioProcess(String),
    #[error("job already active")] JobAlreadyActive { existing_job_id: i64 },
    #[error("internal error")] Internal(#[from] anyhow::Error),
}
```

### 3.2 映射规则

| 变体 | 业务码 | HTTP |
| --- | --- | --- |
| `Unauthorized` | 10001 | 401 |
| `Forbidden` | 10002 | 403 |
| `NotFound` | 20001 | 404 |
| `Validation` | 20002 | 400 |
| `InvalidState` | 20003 | 409 |
| `Duplicate` | 20004 | 409 |
| `ResourceInUse` | 20005 | 409 |
| `UnsupportedFileType` | 30001 | 415 |
| `FileTooLarge` | 30002 | 413 |
| `Storage` | 30003 | 502 |
| `TtsProvider` | 40001 | 502 |
| `TtsRateLimited` | 40002 | 429 |
| `AudioProcess` | 40003 | 500 |
| `JobAlreadyActive` | 40004 | 409（`details.existing_job_id`） |
| `Internal` | 50001 | 500 |

实现 `IntoResponse`：输出统一错误包络，附加 `request_id`；`Internal` 打印 `error!` 与完整错误链，对外只返回通用消息。

## 4. API 层

### 4.1 `router.rs`

职责：声明路由与层叠顺序。

```text
Router
 ├─ /healthz, /readyz（无鉴权）
 ├─ /api/v1/auth/login（限流）
 ├─ /api/v1/*（JWT 中间件）
 ├─ /docs, /api-docs/*（仅非生产）
 └─ fallback → 404 统一错误
```

层叠顺序（由外到内）：

1. `RequestIdLayer`（生成/透传 `X-Request-Id`）
2. `TraceLayer`（记录请求日志与耗时）
3. `CorsLayer`（白名单）
4. `BodyLimitLayer`（默认 2MB，导入/封面路由单独放宽）
5. 路由级 `JwtAuth` 提取器
6. Handler

### 4.2 中间件

| 文件 | 职责 | 关键点 |
| --- | --- | --- |
| `middleware/request_id.rs` | 生成 UUID v7 并注入请求扩展与响应头 | 客户端传入合法 `X-Request-Id` 时复用 |
| `middleware/auth.rs` | 从 `Authorization` 解析 Bearer，校验 JWT | 将 `AuthUser { user_id, role }` 注入扩展 |
| `middleware/rate_limit.rs` | 登录/试听限流 | 内存 `Mutex<HashMap<Ip, Window>>`；单机足够 |

### 4.3 提取器 `extractors.rs`

- `AuthUser`：从扩展取用户，缺失返回 `401`。
- `Pagination`：解析 `page/page_size`，限制 1..=100。
- `JsonBody<T>`：包装 `Json`，将反序列化错误转为 `20002`。

### 4.4 Handler 约定

1. Handler 只做：提取参数 → DTO 校验 → 调用 Service → 转响应。
2. 不写 SQL、不调用 `infra`、不做跨资源业务校验。
3. 所有写接口返回统一包络；`204` 无包络。
4. 异步任务接口返回 `202` + `job_id`。
5. 每个 Handler 标注 `utoipa` 的 `#[utoipa::path]`。

### 4.5 SSE 处理器（`v1/jobs.rs`）

步骤：

1. 校验任务存在。
2. 构造 `Sse` 流：
   - 使用 `tokio_stream::wrappers::IntervalStream`（800ms）；
   - 每 tick 查询 `jobs` 快照；
   - 与上次 `(status, progress, done_items, failed_items, current_step)` 比较，有变化则发送 `progress`；
   - 终态发送 `done` 后 `break`；
   - 每 15s 插入注释心跳 `: ping`；
   - 30min 超时结束。
3. 设置响应头：`Cache-Control: no-cache`、`X-Accel-Buffering: no`。
4. 客户端断开由流 drop 自动清理。

### 4.6 验收点

1. 未带 Token 访问业务接口返回 `401/10001`。
2. 超大请求体返回 `413`，非 JSON 返回 `400/20002`。
3. 所有响应都有 `request_id`，日志可按其串联。
4. SSE 在任务结束后自动关闭，不泄漏连接。

## 5. Domain 层

### 5.1 模型

- `models/*.rs`：与数据库行一一对应的纯结构体（`serde::Serialize` + 可选 `Deserialize`）。
- 时间字段使用 `chrono::DateTime<Utc>`。
- JSON 字段使用 `serde_json::Value` 或强类型结构。
- 不在模型上实现业务方法（避免贫血模型与框架耦合争议，MVP 采用事务脚本式 Service）。

### 5.2 枚举 `enums.rs`

```rust
pub enum BookStatus { Draft, Importing, Parsing, Segmenting, Ready, Failed, Archived }
pub enum ChapterStatus { Pending, Segmenting, Segmented, TtsPartial, TtsDone, Merged, Failed }
pub enum SegmentStatus { Pending, Synthesizing, Done, Failed, Skipped }
pub enum SegmentType { Narration, Dialogue }
pub enum JobType { ImportParse, CharacterDetect, ChapterTts, ChapterMerge, BookExport }
pub enum JobStatus { Queued, Running, Succeeded, Partial, Failed, Canceled }
pub enum JobItemStatus { Pending, Running, Done, Failed, Skipped }
pub enum ExportStatus { Queued, Running, Succeeded, Failed }
pub enum FileKind { Source, Cover, SegmentAudio, ChapterAudio, ExportZip, VoiceSample }
```

每个枚举实现：

- `as_str()` / `from_str()`；
- `sqlx::Type`（或统一用 `String` 在仓储层转换）；
- `Display`。

### 5.3 仓储 trait 示例

```rust
#[async_trait::async_trait]
pub trait BookRepo: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<Book>>;
    async fn list(&self, q: &BookQuery) -> Result<Page<Book>>;
    async fn create(&self, book: &NewBook) -> Result<i64>;
    async fn update(&self, id: i64, patch: &BookPatch, tx: &mut Tx) -> Result<Book>;
    async fn soft_delete(&self, id: i64) -> Result<()>;
    async fn stats(&self, id: i64) -> Result<BookStats>;
}
```

约定：

1. 仓储只做数据访问，不做状态校验。
2. 需要事务的方法显式接收 `&mut sqlx::Transaction`。
3. 查询返回 `Option`，由 Service 转 `NotFound`。
4. 批量操作提供 `insert_many` / `update_many`，减少往返。

### 5.4 外部端口 trait

```rust
#[async_trait::async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put(&self, key: &str, bytes: Bytes, mime: &str) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Bytes>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn head(&self, key: &str) -> Result<Option<ObjectMeta>>;
    async fn ensure_bucket(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait TtsProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>>;
    async fn synthesize(&self, req: &SynthesisRequest) -> Result<SynthesizedAudio>;
}

#[async_trait::async_trait]
pub trait AudioProcessor: Send + Sync {
    async fn probe(&self, path: &Path) -> Result<AudioMeta>;
    async fn merge_with_loudnorm(&self, input: &MergeInput) -> Result<AudioMeta>;
    async fn generate_silence(&self, path: &Path, duration_ms: u64, format: &AudioFormat) -> Result<()>;
    async fn write_tags(&self, path: &Path, tags: &AudioTags) -> Result<()>;
}

#[async_trait::async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, tx: &mut Tx, job: &NewJob, items: &[NewJobItem]) -> Result<i64>;
    async fn claim_next(&self, worker_id: &str) -> Result<Option<Job>>;
    async fn heartbeat(&self, job_id: i64, worker_id: &str) -> Result<bool>;
    async fn complete(&self, job_id: i64, outcome: &JobOutcome) -> Result<()>;
    async fn reclaim_stale(&self, stale_secs: u64) -> Result<u64>;
}
```

## 6. Service 层

### 6.1 `auth_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `login(username, password, ip)` | 1. 限流检查 → 2. 查用户 → 3. Argon2 校验 → 4. 更新 `last_login_at` → 5. 签发 JWT | `10003`、`10004` |
| `me(user_id)` | 查询用户并脱敏 | `404` |
| `change_password(user_id, old, new)` | 1. 校验旧密码 → 2. 校验新密码长度 ≥8 → 3. Argon2 重新哈希 → 4. 更新 | `10003`、`20002` |

JWT Claims：`{ sub: user_id, username, role, iat, exp, jti }`；`jti` 用于日志排查。

### 6.2 `book_service.rs`

| 方法 | 关键步骤 | 错误 |
| --- | --- | --- |
| `list(query)` | 组装过滤条件 → 仓储分页 → 转摘要 DTO（含封面 URL） | — |
| `create(input, user_id)` | 校验标题 → 检查重名 → 写入默认 settings → 返回 | `20004` |
| `get(id)` | 查询未删除书籍 → 聚合旁白音色信息 | `404` |
| `update(id, patch)` | 1. 校验状态（导入中禁止改 settings）→ 2. 若 `default_narrator_voice_id` 或 settings 变更：在同一事务中失效受影响段落与全部章节音频 → 3. 更新书籍 | `20003` |
| `delete(id)` | 1. 检查无进行中任务 → 2. 软删除 | `20003` |
| `upload_cover(id, bytes, mime)` | 1. 校验类型/大小 → 2. SHA-256 → 3. 上传 `covers/...` → 4. 写 files → 5. 更新 `cover_file_id` | `30001/30002/30003` |
| `stats(id)` | 聚合 chapters/segments 统计 + 进行中任务数 | `404` |

失效实现（`narrator voice` 变更）：

```sql
UPDATE segments s
JOIN chapters c ON c.id = s.chapter_id
SET s.status='pending', s.audio_file_id=NULL, s.audio_duration_ms=NULL,
    s.cache_key=NULL, s.attempts=0, s.error_message=NULL
WHERE c.book_id = ?
  AND s.seg_type='narration'
  AND s.voice_id IS NULL
  AND c.narrator_voice_id IS NULL
  AND s.status='done';

UPDATE chapters SET audio_file_id=NULL, audio_duration_ms=NULL, audio_rev=audio_rev+1
WHERE book_id=? AND audio_file_id IS NOT NULL;
```

### 6.3 `import_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `import_txt(file, filename, title, author, user_id)` | 1. 校验扩展名/大小 → 2. SHA-256 → 3. 先创建 book（`draft`）拿 ID → 4. 上传源文件 → 5. 写 files → 6. 事务更新 book（`importing`、`source_file_id`）+ 创建 job → 7. 返回 | `30001/30002/30003` |
| `reimport(book_id, user_id)` | 1. 校验书籍与现有任务 → 2. 创建新 job（`payload.reimport=true`） | `20003`、`40004` |

注意：源文件上传在事务外；若事务失败，尽力删除对象。

### 6.4 `chapter_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `list(book_id, query)` | 分页查询，附带 `segment_count`（子查询或冗余统计） | `404` |
| `get(id)` | 返回含 `raw_text` 的详情 | `404` |
| `update(id, patch)` | 1. 校验章节 → 2. `raw_text` 变更前置条件（无 done 段、无音频）→ 3. 更新标题/正文 → 4. 标题变更则置空章节音频并 `audio_rev+1`；正文变更置 `segmented` 并清空音频 | `20003` |
| `resegment(id, confirm, max_chars)` | 见流程 B-2；单事务删除旧段 + 插入新段 + 更新统计 | `20002/20003` |
| `invalidate_audio(id, tx)` | 置空 `audio_file_id`、`audio_duration_ms`、`audio_rev+1`、状态回退 | — |

### 6.5 `segment_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `list_by_chapter(chapter_id, filters)` | 查询段落 + 批量解析 `resolved_voice`（避免 N+1：一次取书籍/章节/角色/音色） | `404` |
| `update(id, patch)` | 校验状态（非 `synthesizing`）→ 事务更新（若变更 `speaker_id` 则置 `speaker_confidence=1.000`）→ 失效段落 → 失效章节音频 → 重算章节统计 | `20003/20002` |
| `batch_update(items)` | 1. 校验数量与重复 ID → 2. 单事务逐条更新 → 3. 收集受影响章节 → 4. 统一失效章节音频 → 5. 提交 | `20002/20003` |
| `recalc_chapter_stats(chapter_id, tx)` | 统计 done 数，更新 `chapters.tts_done_segments` 与状态 | — |
| `invalidate_segment_audio(id, tx)` | 置 `pending` 并清空音频字段 | — |

### 6.6 `speaker_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `detect_book(book_id, overwrite, user_id)` | 1. 校验书籍 → 2. 检查进行中任务 → 3. 创建 `character_detect` job → 4. 返回 job_id | `40004` |
| `run_detection(job_id, book_id, overwrite, cancel)` | 由 Worker 调用：见流程 C 全部步骤；按章节分批事务写入 | — |
| `resolve_candidates(text)` | 纯函数：候选名收集 + 停用词过滤 + 频次统计 | — |

### 6.7 `character_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `list(book_id)` | 角色列表 + 每个角色段落数 | — |
| `create(book_id, input)` | 校验重名（同书）→ 插入 → 返回 | `20004` |
| `update(id, patch)` | 1. 查旧值 → 2. 更新 → 3. 若 `default_voice_id` 变更：失效该角色无覆盖的 done 段 + 章节音频 | `404/20004` |
| `delete(id)` | 1. 查受影响段落 → 2. 删除角色（FK 置空）→ 3. 失效这些段落 + 章节音频 | `404` |

### 6.8 `voice_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `list(query)` | 分页查询 `voices` | — |
| `sync(provider)` | 1. 查 Registry → 2. `list_voices()` → 3. upsert → 4. 返回数量 | `40001`、`20002` |
| `preview(voice_id, params)` | 1. 校验文本 ≤100 → 2. 解析 Provider → 3. 同步合成 → 4. 返回字节 | `40001/40002` |
| `get_sample(voice_id)` | 1. 查 `sample_file_id` → 2. 有则返回 → 3. 无则生成并缓存 → 4. 返回 | `404/40001` |
| `resolve_voice(segment, chapter, book, characters, voices)` | 按优先级返回 `ResolvedVoice`（含来源标记） | — |

`ResolvedVoice`：

```rust
pub struct ResolvedVoice {
    pub voice_id: i64,
    pub provider: String,
    pub provider_voice_id: String,
    pub display_name: String,
    pub source: VoiceSource, // SegmentOverride / CharacterDefault / ChapterNarrator / BookNarrator / SystemDefault / Fallback
}
```

### 6.9 `tts_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `create_chapter_job(chapter_id, mode, force, user_id)` | 见流程 E.3（含去重、缓存预查、创建 job/items） | `40004/20003` |
| `create_segment_job(segment_id, force, user_id)` | 1. 校验段落 → 2. 去重（同段进行中任务）→ 3. 创建单子项任务 | `40004/20003` |
| `compute_cache_key(...)` | 纯函数，返回 SHA-256 hex | — |
| `apply_cached_audio(segment_id, cache_entry, tx)` | 关联缓存文件、置 done、更新统计、`hit_count+1` | — |
| `recalc_progress(job_id)` | 读取 `job_items` 统计并更新 `jobs`（Worker 节流调用） | — |

### 6.10 `audio_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `create_merge_job(chapter_id, params, user_id)` | 见流程 F.3：校验段落状态 → 合并参数 → 去重 → 建 job | `20003/40004/20002` |
| `invalidate_chapter_audio(chapter_id, tx)` | 置空音频、`audio_rev+1`、状态回退 | — |
| `merge_params(book, request)` | 合并并校验参数范围，返回强类型 `MergeParams` | `20002` |

### 6.11 `export_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `create_export(book_id, options, user_id)` | 1. 校验章节范围 → 2. 去重 → 3. 创建 exports + job → 4. 返回 | `20003/40004` |
| `get(id)` | 查询导出记录，生成 `download_url` | `404` |
| `download(id)` | 1. 校验成功状态 → 2. 取 files → 3. 返回文件流与文件名 | `20003/404` |
| `sanitize_name(title, idx)` | 纯函数：非法字符替换、截断、去重后缀 | — |

### 6.12 `job_service.rs`

| 方法 | 步骤 | 错误 |
| --- | --- | --- |
| `list(query)` | 分页 + 过滤 | — |
| `get(id, include_items)` | 查询任务；可选子项（≤500） | `404` |
| `cancel(id)` | 按状态机处理：queued → canceled；running → 置标记 | `20003` |
| `retry(id, failed_only)` | 校验 `failed/partial` → 复制 payload → 创建新 job/items → 返回新 ID | `20003` |

## 7. Infrastructure 层

### 7.1 MySQL 仓储（`infra/db/repos/`）

通用规则：

1. 使用 `sqlx::query_as` / `query`，所有参数绑定，不拼 SQL 字符串。
2. 动态查询（列表筛选）使用 `QueryBuilder`，仍绑定参数。
3. 分页统一 `LIMIT ? OFFSET ?`，并单独 `COUNT(*)`。
4. 所有时间用 `UTC_TIMESTAMP(3)` 或应用传入 UTC。
5. JSON 字段使用 `sqlx::types::Json`。
6. 每个仓储方法有对应测试（见 07）。

关键查询：

| 场景 | SQL 要点 |
| --- | --- |
| 章节列表 + 段落数 | 左连接聚合或 `chapters.tts_done_segments/segment_count` 冗余字段 |
| 段落列表 | `WHERE chapter_id=? ORDER BY idx`，一次取 ≤2000 |
| 缓存查询 | `SELECT ... FROM audio_cache WHERE cache_key=?` |
| 任务认领 | `FOR UPDATE SKIP LOCKED` |
| 章节统计 | `SELECT status, COUNT(*) FROM segments WHERE chapter_id=? GROUP BY status` |
| 输出音色解析 | 一次联表取 chapter/book/character/voice，避免逐段查询 |

### 7.2 RustFS 存储（`infra/storage/rustfs.rs`）

实现要点：

1. `aws_sdk_s3::Client` 配置：
   - `endpoint_url(config.rustfs.endpoint)`；
   - `force_path_style(true)`；
   - 静态凭证；
   - `region` 使用配置值。
2. `ensure_bucket`：`head_bucket` 失败则 `create_bucket`，已存在视为成功。
3. `put`：流式 `ByteStream::from(bytes)`，设置 `content_type`。
4. `get`：`collect()` 为 `Bytes`（MVP 段落音频通常 <1MB，章节 <100MB；章节下载可后续改流式）。
5. `delete`：对象不存在视为成功（幂等）。
6. 错误统一转 `AppError::Storage`，日志记录 `key/status_code`。
7. 测试实现 `memory.rs`：`Mutex<HashMap<String, Bytes>>`。

### 7.3 TTS（`infra/tts/`）

#### Registry

```rust
pub struct TtsRegistry {
    providers: HashMap<&'static str, Arc<dyn TtsProvider>>,
    default_provider: &'static str,
    semaphores: HashMap<&'static str, Arc<Semaphore>>,
    min_interval: Duration,
}
```

- `provider(name)` 返回实现，未知返回 `20002`。
- 每次合成前获取 Provider 信号量并按 `min_interval` 节流。
- 默认 `edge`。

#### Edge Provider 实现步骤

1. `list_voices()`：
   1. `GET https://speech.platform.bing.com/consumer/speech/synthesize/readaloud/voices/list?trustedclienttoken={TOKEN}`；
   2. 超时 15s；
   3. 解析 JSON 数组 → `VoiceInfo { id, name, locale, gender, styles }`；
   4. 失败转 `TtsProvider`。
2. `synthesize(req)`：
   1. XML 转义文本；
   2. 构造 SSML（`speak/voice/prosody`）；
   3. `tokio-tungstenite` 连接 Edge WSS（超时 10s）；
   4. 发送配置帧 + SSML 帧；
   5. 循环读取二进制帧，累积音频字节；文本帧中的 `turn.end` 结束；
   6. 总超时 `TTS_TIMEOUT_SECS`；
   7. 返回 `SynthesizedAudio { bytes, format, provider_version }`；
   8. 403/429/网络错误映射为可重试错误，其他为不可重试。
3. 文本切分：>1000 字符按句末标点切分，逐段合成后拼接 MP3 字节（帧级拼接，音色一致）。
4. 常量：
   - `EDGE_TRUSTED_CLIENT_TOKEN`（公开客户端 Token，单独常量便于更新）；
   - `EDGE_WS_URL`；
   - `EDGE_OUTPUT_FORMAT=audio-24khz-48kbitrate-mono-mp3`；
   - `provider_version="edge-v1"`。

#### Mock Provider

- 返回 `fixtures/audio/sample_segment.mp3` 的字节。
- `list_voices` 返回固定 2 个音色。
- 可配置失败次数，用于测试重试。

### 7.4 音频（`infra/audio/`）

#### `probe.rs`

- 调用 `ffprobe -v error -show_entries format=duration,size -of json <path>`。
- 解析秒数 → 毫秒。
- 超时 15s；失败转 `AudioProcess`。

#### `ffmpeg.rs`

- `merge_with_loudnorm(input)`：
  1. 校验 `concat.txt` 路径在临时目录内；
  2. 构造参数数组（不经过 shell）；
  3. `tokio::process::Command` 启动，捕获 stdout/stderr；
  4. 超时 600s，超时 `kill()`；
  5. 非零退出 → `AudioProcess`（stderr 截断 2KB 入日志）；
  6. 成功返回 `AudioMeta`。
- `generate_silence(path, ms, format)`：
  - `-f lavfi -i anullsrc=r={sample_rate}:cl={mono|stereo} -t {secs} -c:a libmp3lame -b:a {bitrate}`。
- `write_tags(path, tags)`：
  - 使用 `-metadata` 在合并时一次写入，不二次 remux（避免重复编码）。
- 所有命令执行前校验可执行文件存在（`/readyz` 也检查）。

#### 并发

- `FfmpegGuard`：全局 `Semaphore(FFMPEG_CONCURRENCY)`，任务结束释放。

### 7.5 队列（`infra/queue/mysql_queue.rs`）

| 方法 | 步骤 |
| --- | --- |
| `enqueue` | 在传入事务中插入 `jobs` → 批量插入 `job_items` → 返回 job_id |
| `claim_next` | 事务：`SELECT ... FOR UPDATE SKIP LOCKED` → 条件 UPDATE → 提交 → 返回 Job |
| `heartbeat` | `UPDATE ... WHERE id=? AND status='running' AND worker_id=?`，返回 `rows_affected>0` |
| `complete` | 事务更新 `jobs.status/result/error/finished_at` + `worker_id=NULL` |
| `reclaim_stale` | 见流程 H.5，返回回收数量 |

错误处理：数据库错误统一 `AppError::Internal`；认领冲突返回 `None`，不视为错误。

## 8. Worker 层

### 8.1 Dispatcher

```rust
pub struct Dispatcher {
    state: Arc<WorkerState>,
    handlers: HashMap<JobType, Arc<dyn JobHandler>>,
    semaphore: Arc<Semaphore>, // WORKER_CONCURRENCY
}
```

主循环步骤：

1. 注册 Handler：`import_parse`、`character_detect`、`chapter_tts`、`chapter_merge`、`book_export`。
2. 循环直到收到停机信号：
   1. 每 30s 执行一次 `reclaim_stale`；
   2. 获取信号量许可；
   3. `claim_next(worker_id)`；
   4. `Some(job)` → `spawn(handle_job(job))`；`None` → sleep 1s；
   5. 收到停机信号 → 停止认领，等待运行任务（≤30s）。
3. `handle_job`：
   1. 为任务创建 `CancellationToken`；
   2. 启动心跳任务（10s），失败时取消任务；
   3. 调用对应 Handler；
   4. Handler 返回 `JobOutcome`；
   5. `queue.complete()`；
   6. 取消心跳并 join；
   7. 记录耗时与结果日志。

### 8.2 JobContext 与 Progress

```rust
pub struct JobContext {
    pub job: Job,
    pub worker_id: String,
    pub cancel: CancellationToken,
    pub progress: ProgressReporter,
}
```

`ProgressReporter`：

- 维护 `done/failed/skipped/total` 原子计数；
- 每 500ms 最多写一次数据库（节流）；
- 每次写入更新 `progress/current_step/updated_at`；
- 任务结束强制 flush。

### 8.3 Handlers

| Handler | 入口 | 主要调用 | 输出 |
| --- | --- | --- | --- |
| `import_parse.rs` | `handle(ctx)` | `import_service` 的解析函数 + `speaker_service.run_detection` | 章节/段落/角色统计 |
| `character_detect.rs` | `handle(ctx)` | `speaker_service.run_detection` | 识别统计 |
| `chapter_tts.rs` | `handle(ctx)` | `tts_service` + `TtsProvider` + `ObjectStorage` | 完成/失败子项 |
| `chapter_merge.rs` | `handle(ctx)` | `ObjectStorage` + `AudioProcessor` | 章节音频 file_id/时长 |
| `book_export.rs` | `handle(ctx)` | `ObjectStorage` + `zip` | 导出 file_id |

Handler 约定：

1. 不直接构造 HTTP 错误；失败返回 `JobOutcome { status: Failed, error }`。
2. 每个循环迭代检查 `ctx.cancel.is_cancelled()` 与数据库 `cancel_requested`。
3. 所有临时文件使用 `TempDirGuard`（Drop 时清理）。
4. 结果 JSON 写入 `jobs.result`，供 API 返回。

## 9. 工具模块

### 9.1 `utils/encoding.rs`

```rust
pub enum TextEncoding { Utf8, Utf8Bom, Gb18030 }

pub fn decode_text(bytes: &[u8]) -> Result<(String, TextEncoding), DecodeError>;
```

步骤：

1. BOM 检测；
2. 严格 UTF-8；
3. GB18030 解码；
4. 替换字符比例检查（>0.5% 失败）。

### 9.2 `utils/text.rs`

| 函数 | 说明 |
| --- | --- |
| `normalize_text` | 换行、零宽字符、行尾空白、空行压缩 |
| `split_chapters` | 主正则 + 特殊标题 + 长度兜底；返回 `Vec<RawChapter>` |
| `split_segments` | 按空行、句末标点、`max_chars` 切分 |
| `classify_paragraph` | 返回 `Vec<(String, SegmentType)>`，处理引号拆分 |
| `extract_quote_spans` | 识别中文引号对，返回字节区间 |
| `sanitize_filename` | ZIP 文件名清洗与截断 |
| `truncate_chars` | 按字符截断，不破坏 UTF-8 |

所有函数必须是无 I/O 的纯函数，便于单测。

### 9.3 `utils/hash.rs`

- `sha256_hex(bytes)`；
- `cache_key(parts)`（固定分隔符）；
- `short_hash(hex, n)`。

### 9.4 `utils/id.rs`

- `request_id()`：UUID v7；
- `object_key(kind, ...)`：统一生成对象 Key；
- `worker_id()`：`{hostname}-{pid}-{uuid8}`。

## 10. 事务、并发与错误处理规范

| 规范 | 内容 |
| --- | --- |
| 事务边界 | 只在 Service 开启；跨仓储操作共用一个事务 |
| 长时间操作 | 下载/合成/FFmpeg 不持有数据库事务；先完成后短事务落库 |
| 并发控制 | 段落状态 CAS；任务 `SKIP LOCKED`；FFmpeg/Provider 信号量 |
| 幂等 | 任务重试按状态跳过已完成项；文件写入使用确定性 Key |
| 超时 | Provider 45s；ffprobe 15s；FFmpeg 600s；HTTP 请求 30s |
| 重试 | Provider/上传重试 2–3 次；业务校验失败不重试 |
| 日志 | 禁止输出密钥、密码、JWT；文本日志截断 100 字 |
| panic | Handler 不 panic；使用 `Result`；SSE/心跳任务内 panic 不得影响主进程 |
| 清理 | `TempDirGuard` 保证任务结束删除临时目录；启动时清理 24h 前残留 |

## 11. 模块验收点

1. `cargo clippy -- -D warnings` 无告警。
2. Service 层单测覆盖：状态机校验、缓存键、音色优先级、失效规则。
3. API 集成测试覆盖：登录、书籍 CRUD、导入、合成（Mock Provider）、拼接、导出。
4. Worker 测试覆盖：认领、心跳、回收、取消。
5. 日志中可通过 `request_id` 或 `job_id` 串联一次完整请求。
