# 06 后端功能模块详细设计（按功能组织）

> 文档版本：v0.3（评审稿）
> **本版变更：废弃 DDD 分层设计，改为按功能模块组织；补充回收站模块说明。**
> 每个功能目录自带 `routes / handler / service / repo / model / dto`；共享能力放在 `shared/`（历史文档中的 `common/` 已迁移）。
> 本文件与 [01 目录结构](./01-directory-structure.md)、[03 数据库](./03-database-design.md)、[04 API](./04-api-design.md)、[05 流程](./05-core-workflows.md)、[10 回收站](./10-recycle-bin-design.md) 对应。
> **当前实际目录以源码为准**：`runtime/`、`account/`、`library/`、`text_import/`、`casting/`、`speech/`、`audio/`、`delivery/`、`task_center/`、`recycle_bin/`、`system/`、`shared/`、`worker/`。

## 1. 组织方式

### 1.1 一个功能一个目录

```text
src/features/books/
├── mod.rs
├── routes.rs      # 该功能负责的路由
├── handler.rs     # HTTP 参数提取、校验、调用 service
├── service.rs     # 业务规则、事务、状态流转
├── repo.rs        # books 表的 SQL
├── model.rs       # books 行结构体
└── dto.rs         # 请求/响应
```

功能模块一览：

| 功能 | 目录 | 路由前缀 | 表归属 |
| --- | --- | --- | --- |
| 健康检查 | `features/health` | `/healthz`、`/readyz` | — |
| 认证 | `features/auth` | `/api/v1/auth/*` | `users` |
| 书籍 | `features/books` | `/api/v1/books*` | `books` |
| 导入 | `features/imports` | `/api/v1/books/import` | 无（编排其他功能） |
| 章节 | `features/chapters` | `/api/v1/books/{id}/chapters`、`/chapters/*` | `chapters` |
| 段落 | `features/segments` | `/api/v1/chapters/{id}/segments`、`/segments/*` | `segments` |
| 角色 | `features/characters` | `/api/v1/books/{id}/characters`、`/characters/*` | `characters` |
| 音色 | `features/voices` | `/api/v1/voices/*` | `voices` |
| 合成 | `features/tts` | `/api/v1/chapters/{id}/tts`、`/segments/{id}/tts` | `audio_cache` |
| 音频 | `features/audio` | `/api/v1/chapters/{id}/merge`、`/chapters/{id}/audio` | 无（编排章节/段落/文件） |
| 导出 | `features/exports` | `/api/v1/books/{id}/exports`、`/exports/*` | `exports` |
| 任务 | `features/jobs` | `/api/v1/jobs/*` | `jobs`、`job_items` |
| 文件 | `features/files` | `/api/v1/files/{id}` | `files` |

### 1.2 表归属与写入规则

1. 每张业务表只有一个归属功能，负责该表的增删改查 SQL。
2. 其他功能需要修改归属表时，**调用归属功能的 service 函数**，不直接写 SQL。
3. 跨功能写事务由发起流程的功能开启，将 `&mut Transaction` 传给归属功能的 service。
4. 查询同理：跨功能读取调用归属功能 service 暴露的查询函数。

| 表 | 归属功能 | 其他功能可调用的写函数（示例） |
| --- | --- | --- |
| `users` | `auth` | — |
| `books` | `books` | `update_status`、`update_stats` |
| `chapters` | `chapters` | `bulk_create`、`update_audio`、`invalidate_audio`、`recalc_status` |
| `segments` | `segments` | `bulk_create`、`delete_by_chapter`、`mark_synthesizing/done/failed`、`bulk_set_speakers` |
| `characters` | `characters` | — |
| `voices` | `voices` | — |
| `files` | `files` | `register`、`get` |
| `audio_cache` | `tts` | `get/put` |
| `jobs` / `job_items` | `jobs` | `create/enqueue`、`update_progress`、`complete` |
| `exports` | `exports` | `update_status` |

### 1.3 功能注册

1. 每个功能 `mod.rs` 导出 `routes()`。
2. `src/router.rs` 汇总：

```rust
pub fn build_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::routes())
        .merge(auth::routes())
        .merge(books::routes())
        .merge(imports::routes())
        .merge(chapters::routes())
        .merge(segments::routes())
        .merge(characters::routes())
        .merge(voices::routes())
        .merge(tts::routes())
        .merge(audio::routes())
        .merge(exports::routes())
        .merge(jobs::routes())
        .merge(files::routes())
}
```

3. 各功能 `routes.rs` 只声明自己负责的路径；同一前缀的不同子路径可以安全合并。
4. 新增功能步骤：新建目录 → 实现 `routes/handler/service/repo/model/dto` → 在 `features/mod.rs` 声明 → 在 `router.rs` merge。

## 2. 启动与状态

### 2.1 启动流程

```text
main.rs（API）                          bin/worker.rs（Worker）
    │                                       │
    ├─ 加载配置 config.rs                   ├─ 加载配置
    ├─ 初始化日志                           ├─ 初始化日志
    ├─ common::db::connect + migrate        ├─ common::db::connect
    ├─ bootstrap（管理员/桶/音色种子）        ├─ 构建 WorkerState
    ├─ 构建 AppState                        └─ Dispatcher::run()
    └─ axum::serve(app::build())
```

### 2.2 `AppState`

```rust
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: MySqlPool,
    pub storage: Arc<Storage>,       // common/storage.rs
    pub tts: Arc<TtsRegistry>,       // features/tts/provider/mod.rs
    pub ffmpeg: Arc<Ffmpeg>,         // common/ffmpeg.rs
}
```

`WorkerState`：

```rust
pub struct WorkerState {
    pub app: Arc<AppState>,
    pub worker_id: String,
}
```

约定：

1. 服务函数统一接收 `&AppState`（或其中所需字段），不创建全局单例。
2. 不使用数据访问 trait 聚合、不使用依赖注入容器。
3. API 与 Worker 共用同一套功能 service，不复制业务逻辑。
4. 测试构造 `AppState` 时替换为测试数据库、内存 Storage、Mock TTS Provider。

## 3. 路由与中间件

### 3.1 中间件

| 文件 | 职责 |
| --- | --- |
| `middleware/request_id.rs` | 生成/透传 `X-Request-Id`（UUID v7） |
| `middleware/auth.rs` | 解析 Bearer Token，注入 `AuthUser` |
| `middleware/rate_limit.rs` | 登录（10 次/分钟/IP）、试听（20 次/分钟/IP） |

挂载顺序（外到内）：

```text
RequestId → Trace → CORS → BodyLimit → 路由级 JWT → Handler
```

### 3.2 Handler 约定

1. 只做参数提取、DTO 校验、调用 service、构造响应。
2. 不写 SQL、不调用其他功能的 repo、不做跨资源业务判断。
3. 异步任务接口返回 `202 + job_id`。
4. 所有错误转 `AppError`，由统一 `IntoResponse` 输出错误包络。
5. 每个 Handler 标注 `utoipa` 路径与 schema。

### 3.3 Service 约定

1. 一个业务动作一个公开函数，例如 `books::service::create`。
2. 函数签名示例：

```rust
pub async fn create(
    state: &AppState,
    input: CreateBookInput,
    user_id: i64,
) -> Result<Book, AppError>;
```

3. 需要事务时在 service 内开启，`repo` 函数接收 `&mut Transaction`。
4. 跨功能调用只调用对方 `service`。
5. 纯算法（切章、切段、识别规则、缓存键）放在功能内独立文件，便于单测。

### 3.4 Repo 约定

1. 只做 SQL，不含业务判断。
2. 函数接收 `&MySqlPool` 或 `&mut Transaction<'_, MySql>`。
3. 动态筛选使用 `QueryBuilder` 绑定参数，不拼字符串。
4. 返回 `Option`，由 service 转 `NotFound`。

## 4. 功能模块设计

### 4.1 `health` 健康检查

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/healthz` | GET | 进程存活 |
| `/readyz` | GET | MySQL、RustFS、FFmpeg 可用性 |

步骤（`/readyz`）：

1. `SELECT 1` 检查 MySQL。
2. `head_bucket` 检查 RustFS 桶。
3. 执行 `ffmpeg -version`（缓存结果 30s）检查可执行文件。
4. 全部通过 → 200 `{status:"ok", checks:{...}}`；任一失败 → 503。

### 4.2 `auth` 认证

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/auth/login` | POST | 登录 |
| `/api/v1/auth/me` | GET | 当前用户 |
| `/api/v1/auth/password` | POST | 修改密码 |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::login` | 1. 限流检查 → 2. `repo::find_by_username` → 3. Argon2 校验 → 4. 更新 `last_login_at` → 5. 签发 JWT | `10003/10004` |
| `service::me` | 查询用户并脱敏 | `404` |
| `service::change_password` | 1. 校验旧密码 → 2. 新密码长度 ≥8 → 3. 重新哈希 → 4. 更新 | `10003/20002` |

JWT Claims：`{ sub, username, role, iat, exp, jti }`。

### 4.3 `books` 书籍

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/books` | GET / POST | 列表 / 创建 |
| `/api/v1/books/{id}` | GET / PATCH / DELETE | 详情 / 更新 / 软删除 |
| `/api/v1/books/{id}/cover` | POST | 上传封面 |
| `/api/v1/books/{id}/stats` | GET | 统计 |

关键函数：

| 函数 | 关键步骤 | 错误 |
| --- | --- | --- |
| `service::list` | 组装筛选 → `repo::list` → 转摘要 DTO | — |
| `service::create` | 校验标题 → 重名检查 → 默认 settings → `repo::insert` | `20004` |
| `service::get` | 查询未删除书籍 → 附封面 URL | `404` |
| `service::update` | 1. 导入中禁止改 settings → 2. 旁白音色变更：调用 `segments::service::invalidate_narration_for_book` → 3. settings 变更：调用 `chapters::service::invalidate_all_audio` → 4. `repo::update` | `20003` |
| `service::delete` | 1. 检查进行中任务（`jobs::service::count_active`）→ 2. 软删除 | `20003` |
| `service::upload_cover` | 1. 校验类型/大小 → 2. `storage.put` → 3. `files::service::register` → 4. 更新 `cover_file_id` | `30001/30002/30003` |
| `service::stats` | `repo` 聚合 + `jobs::service::count_active` | `404` |

不直接操作 `chapters/segments` 表；失效逻辑通过对应功能 service 完成。

### 4.4 `imports` 导入解析

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/books/import` | POST | 上传 TXT 并创建导入任务 |

文件组成：

| 文件 | 职责 |
| --- | --- |
| `parser.rs` | 导入流水线编排：调用 `common::encoding` 解码、`common::text` 清洗/切章/切段，返回 `ParsedBook` |
| `service.rs` | 创建导入任务；Worker 调用 `run_import` 执行流水线 |

关键函数：

| 函数 | 步骤 |
| --- | --- |
| `service::create_import` | 1. 校验扩展名/大小 → 2. 计算 SHA-256 → 3. `books::service::create_draft` 拿 `book_id` → 4. `storage.put` 源文件 → 5. `files::service::register` → 6. 事务内更新书籍为 `importing` + `jobs::service::create`（`import_parse`）→ 7. 返回 `{book_id, job_id}` |
| `service::run_import` | 1. 读 `file_id` → 2. `storage.get` → 3. `common::encoding::decode_text` → 4. `common::text::normalize` → 5. `common::text::split_chapters` / `split_segments` → 6. 更新进度 → 7. 逐章事务：`chapters::service::bulk_create` + `segments::service::bulk_create` → 8. 更新书籍统计 → 9. 可选调用 `characters::service::run_detect` → 10. 书籍 `ready`、任务完成 |

`parser.rs` 编排函数：

```rust
pub struct ParsedBook {
    pub chapters: Vec<ParsedChapter>,
}

pub struct ParsedChapter {
    pub index: u32,
    pub title: String,
    pub raw_text: String,
    pub char_count: u32,
    pub segments: Vec<ParsedSegment>,
}

pub fn parse_book(bytes: &[u8], max_chars: usize) -> Result<ParsedBook, ParseError>;
```

算法本身放在 `common/encoding.rs` 与 `common/text.rs`，`parser.rs` 只做导入流程的组装，便于单测与复用。

错误：编码不支持、章节数为 0、单章入库失败 → 任务 `failed`，书籍 `failed`，记录章节序号。

### 4.5 `chapters` 章节

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/books/{id}/chapters` | GET | 章节分页列表 |
| `/api/v1/chapters/{id}` | GET / PATCH | 详情 / 编辑 |
| `/api/v1/chapters/{id}/resegment` | POST | 重新切段（同步） |

关键函数：

| 函数 | 关键步骤 | 错误 |
| --- | --- | --- |
| `service::list` | 分页查询 + 段落数 | `404` |
| `service::get` | 返回含 `raw_text` 的详情 | `404` |
| `service::update` | 1. 正文修改前置条件（无 done 段、无音频）→ 2. 更新标题/正文 → 3. 标题变更则调用 `service::invalidate_audio`；正文变更置 `segmented` | `20003` |
| `service::resegment` | 1. 校验 `confirm` 与前置条件 → 2. 事务：`segments::service::delete_by_chapter` + `common::text::split_segments` 切段 + `segments::service::bulk_create` → 3. 更新章节状态、清空音频 → 4. 提交 | `20002/20003` |
| `service::invalidate_audio` | 置空 `audio_file_id/audio_duration_ms`、`audio_rev+1`、状态回退 | — |
| `service::invalidate_all_audio` | 按 `book_id` 批量失效章节音频（供 books 调用） | — |
| `service::recalc_status` | 根据段落统计更新 `tts_done_segments` 与章节状态（供 segments/tts/audio 调用） | — |
| `service::list_merged_by_book` | 返回已拼接章节（供 exports 调用） | — |

### 4.6 `segments` 段落

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/chapters/{id}/segments` | GET | 段落列表（≤2000） |
| `/api/v1/segments/{id}` | PATCH | 编辑单段 |
| `/api/v1/segments/batch` | PATCH | 批量编辑（≤200） |

关键函数：

| 函数 | 关键步骤 | 错误 |
| --- | --- | --- |
| `service::list_by_chapter` | 查询段落 → 调 `voices::service::resolve_for_segments` 批量解析音色 | `404` |
| `service::update` | 1. 非 `synthesizing` → 2. 更新字段（说话人变更置 `confidence=1.000`）→ 3. 需要失效时清空音频与缓存键 → 4. 调 `chapters::service::invalidate_audio` + `recalc_status` | `20003/20002` |
| `service::batch_update` | 单事务逐条执行 update；收集章节统一失效 | `20002/20003` |
| `service::bulk_create` | 导入/重切时批量插入（接收 `&mut Transaction`） | — |
| `service::delete_by_chapter` | 删除章节全部段落 | — |
| `service::mark_synthesizing` | CAS：`pending/failed → synthesizing`，返回是否抢占成功 | — |
| `service::mark_done` | 写音频文件、时长、缓存键、状态 | — |
| `service::mark_failed` | 写错误、尝试次数 | — |
| `service::bulk_set_speakers` | 角色识别回填 `speaker_id/confidence` | — |
| `service::invalidate_narration_for_book` | 旁白音色变更时失效受影响段落 | — |
| `service::invalidate_for_character` | 角色音色变更/删除时失效相关段落 | — |

说明：`repo.rs` 只包含 `segments` 表的 SQL，其他功能通过上述 service 函数修改段落。

### 4.7 `characters` 角色与识别

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/books/{id}/characters` | GET / POST | 列表 / 创建 |
| `/api/v1/characters/{id}` | PATCH / DELETE | 更新 / 删除 |
| `/api/v1/books/{id}/characters/detect` | POST | 创建识别任务 |

文件：

| 文件 | 职责 |
| --- | --- |
| `detector.rs` | 候选名收集、归属句式匹配、置信度计算（纯函数） |
| `service.rs` | CRUD、识别编排、失效联动 |
| `repo.rs` | `characters` 表 |

关键函数：

| 函数 | 步骤/说明 | 错误 |
| --- | --- | --- |
| `service::create` | 同书重名校验 → 插入 | `20004` |
| `service::update` | 更新字段；音色变更时调 `segments::service::invalidate_for_character` + `chapters::service::invalidate_all_audio` | `404/20004` |
| `service::delete` | 1. 删除角色（外键置空 speaker）→ 2. 调 `segments::service::invalidate_for_character` | `404` |
| `service::create_detect_job` | 1. 校验书籍 → 2. 去重 → 3. `jobs::service::create`（`character_detect`） | `40004` |
| `service::run_detect` | 1. 加载角色/别名 → 2. 加载段落 → 3. `detector::collect_candidates` → 4. 逐段 `detector::match_speaker` → 5. 创建候选角色 → 6. `segments::service::bulk_set_speakers` → 7. 写任务结果 | — |

`detector.rs` 核心函数：

```rust
pub fn collect_candidates(segments: &[SegmentRow]) -> Vec<Candidate>;
pub fn match_speaker(
    dialogue: &SegmentRow,
    prev: Option<&SegmentRow>,
    next: Option<&SegmentRow>,
    aliases: &AliasMap,
) -> Option<SpeakerGuess>; // 含 name、confidence、pattern
```

识别规则与置信度见 [05 核心流程 C](./05-core-workflows.md)。

### 4.8 `voices` 音色

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/voices` | GET | 音色分页列表 |
| `/api/v1/voices/sync` | POST | 同步 Provider 音色目录 |
| `/api/v1/voices/preview` | POST | 文本试听（同步） |
| `/api/v1/voices/{id}/sample` | GET | 音色样例音频 |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::list` | 分页查询 | — |
| `service::sync` | 1. 从 `state.tts` 取 Provider → 2. `list_voices()` → 3. 按 `(provider, provider_voice_id)` upsert | `40001` |
| `service::preview` | 1. 文本 ≤100 字 → 2. 调 Provider 同步合成 → 3. 返回 MP3 字节 | `40001/40002` |
| `service::get_sample` | 1. 查 `sample_file_id` → 2. 无则合成并 `files::service::register` → 3. 返回 | `404/40001` |
| `service::resolve_voice` | 按优先级返回 `ResolvedVoice`（段落覆盖 > 角色 > 章节旁白 > 书籍旁白 > 系统默认） | — |
| `service::resolve_for_segments` | 批量解析，避免逐段查询 | — |

`repo.rs` 只包含 `voices` 表 SQL；样例文件写入通过 `files::service`。

### 4.9 `tts` 合成

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/chapters/{id}/tts` | POST | 整章批量合成任务 |
| `/api/v1/segments/{id}/tts` | POST | 单段合成任务 |

文件：

| 文件 | 职责 |
| --- | --- |
| `service.rs` | 创建任务、筛选候选、缓存键、进度、Worker 执行入口 |
| `cache.rs` | `audio_cache` 读写、缓存键计算 |
| `repo.rs` | `audio_cache` 表 SQL |
| `provider/mod.rs` | `TtsProvider` trait + `TtsRegistry`（功能内多引擎抽象） |
| `provider/openai.rs` | OpenAI 兼容 `/audio/speech`（OpenAI / 阶跃星辰） |
| `provider/minimax.rs` | MiniMax `t2a_v2` 实现 |
| `provider/mock.rs` | 测试实现（后续补） |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::create_chapter_job` | 1. 校验章节 → 2. 去重 → 3. 按 mode 筛选段落 → 4. 计算缓存键并预查缓存 → 5. 事务：`jobs::service::create` + `job_items` 批量插入 | `40004/20003` |
| `service::create_segment_job` | 1. 校验段落 → 2. 去重 → 3. 创建单子项任务 | `40004/20003` |
| `cache::key` | `sha256(provider + version + voice + text + emotion + rate + pitch + volume + format)` | — |
| `cache::get/put` | 查询/写入 `audio_cache` | — |
| `service::run_tts` | Worker 入口：并发池、逐段 CAS、Provider 调用、上传、落库、重试、取消 | — |

`TtsProvider`（放在功能目录内，不叫端口）：

```rust
#[async_trait::async_trait]
pub trait TtsProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, AppError>;
    async fn synthesize(&self, req: &SynthesisRequest) -> Result<SynthesizedAudio, AppError>;
}
```

`TtsRegistry`：

- `HashMap<String, Arc<dyn TtsProvider>>`；
- 默认 `openai`；
- Provider 级信号量与最小请求间隔；
- 未知 Provider 返回 `20002`。

Provider 实现说明：

1. OpenAI 与阶跃星辰共用 OpenAI 兼容 `/audio/speech`，只替换 Base URL、API Key、模型。
2. MiniMax 使用 `/v1/t2a_v2`，返回 hex 音频，服务端解码为 MP3。
3. 三家均不内置音色；`list_voices` 返回空，音色通过 `POST /voices` 手动登记。
4. 403/429/网络错误标记可重试；Key 缺失/音色不存在等标记不可重试。

段落状态写入通过 `segments::service::mark_synthesizing/done/failed`；文件登记通过 `files::service::register`。

### 4.10 `audio` 拼接与音频下载

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/chapters/{id}/merge` | POST | 创建拼接任务 |
| `/api/v1/chapters/{id}/audio` | GET / DELETE | 下载 / 失效章节音频 |
| `/api/v1/segments/{id}/audio` | GET | 下载段落音频 |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::create_merge_job` | 1. 校验段落状态与参数 → 2. 去重 → 3. `jobs::service::create`（`chapter_merge`） | `20003/40004/20002` |
| `service::run_merge` | 1. 建临时目录 → 2. 下载段落音频 → 3. `ffprobe` 校验 → 4. 生成静音 → 5. 写 concat 清单 → 6. `common::ffmpeg::merge_with_loudnorm` → 7. 上传 → 8. `files::service::register` → 9. `chapters::service::update_audio` 与 `books::service::update_stats` | `40003` |
| `service::get_chapter_audio` | 通过 `files::service` 取文件并流式返回 | `20001` |
| `service::delete_chapter_audio` | 调 `chapters::service::invalidate_audio` | `20001` |

不直接写 `chapters` 表；章节音频字段由 `chapters::service::update_audio` 更新。

### 4.11 `exports` 导出

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/books/{id}/exports` | POST | 创建导出任务 |
| `/api/v1/exports/{id}` | GET | 导出详情 |
| `/api/v1/exports/{id}/download` | GET | 下载 ZIP |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::create_export` | 1. 校验章节范围（`chapters::service::list_merged_by_book`）→ 2. 去重 → 3. `repo::insert` + `jobs::service::create` | `20003/40004` |
| `service::get` | 查询导出记录 + 下载 URL | `404` |
| `service::download` | 校验成功 → `files::service::get` → 返回 ZIP | `20003/404` |
| `service::run_export` | 1. 建临时目录 → 2. 下载章节 MP3 → 3. 清洗文件名 → 4. 可选封面 → 5. 生成 `manifest.json` → 6. 流式 ZIP → 7. 上传 → 8. `files::service::register` → 9. 更新导出记录 | `30003/40003` |
| `service::sanitize_name` | 非法字符替换、截断 80 字符、重名后缀 | — |

### 4.12 `jobs` 任务与队列

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/jobs` | GET | 任务分页列表 |
| `/api/v1/jobs/{id}` | GET | 任务详情（可选 items） |
| `/api/v1/jobs/{id}/cancel` | POST | 取消 |
| `/api/v1/jobs/{id}/retry` | POST | 重试（创建新任务） |
| `/api/v1/jobs/{id}/events` | GET | SSE 进度流 |

文件：

| 文件 | 职责 |
| --- | --- |
| `service.rs` | 状态机、取消、重试、进度计算、SSE 数据读取 |
| `queue.rs` | MySQL 入队/认领/心跳/回收 |
| `repo.rs` | `jobs`、`job_items` SQL |
| `model.rs` | Job、JobItem、JobStatus |
| `dto.rs` | 查询与响应 DTO |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::list/get` | 分页/详情，可选子项 ≤500 | `404` |
| `service::cancel` | queued → canceled；running → `cancel_requested=1`；终态 → 409 | `20003` |
| `service::retry` | 校验 `failed/partial` → 复制 payload → 创建新 job/items | `20003` |
| `queue::claim_next` | 事务 `SELECT ... FOR UPDATE SKIP LOCKED` + 条件更新 | — |
| `queue::heartbeat` | 带 `worker_id` 条件更新 | — |
| `queue::reclaim_stale` | 心跳超时 90s 回收 | — |
| `service::progress_sse` | 800ms 轮询快照，变化推送；15s 心跳；终态关闭 | — |

### 4.13 `files` 文件

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/files/{id}` | GET | 鉴权下载（inline/attachment） |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::register` | 1. 写 `files` 表 → 2. 返回 `file_id` | — |
| `service::get_meta` | 查询文件元数据 | `404` |
| `service::download` | 1. 查元数据 → 2. `storage.get` → 3. 设置 MIME/文件名 → 4. 返回字节 | `404/30003` |

### 4.14 `recycle_bin` 回收站

> 完整设计见 [10 回收站设计](./10-recycle-bin-design.md)。

| 路由 | 方法 | 说明 |
| --- | --- | --- |
| `/api/v1/recycle-bin` | GET / DELETE | 列表 / 清空（需 `confirm=true`） |
| `/api/v1/recycle-bin/summary` | GET | 总数、按类型计数、保留期、最早到期 |
| `/api/v1/recycle-bin/{resource_type}/{id}/restore` | POST | 恢复单个 |
| `/api/v1/recycle-bin/{resource_type}/{id}` | DELETE | 彻底删除单个 |
| `/api/v1/recycle-bin/restore` | POST | 批量恢复（≤100） |
| `/api/v1/recycle-bin/purge` | POST | 批量彻底删除（≤100） |

关键函数：

| 函数 | 步骤 | 错误 |
| --- | --- | --- |
| `service::list` | 查询 `deleted_at IS NOT NULL` → 计算到期时间/剩余天数 | — |
| `service::restore_one` | 1. 行锁回收站书籍 → 2. 书名冲突策略 → 3. 清空 `deleted_at/deleted_by` | `404/20004` |
| `service::purge_one` | 1. 行锁 → 2. 检查进行中任务 → 3. 删任务/书籍 → 4. 删除未共享文件与对象 | `404/20003` |
| `service::empty` | 校验 `confirm` → 遍历回收站条目逐个彻底删除 | `400` |
| `service::purge_expired` | Worker 每小时调用，按保留期扫描并彻底删除 | — |

`resource_type` 当前只支持 `book`；书籍进入回收站后，章节/段落/角色查询通过 `JOIN books ... deleted_at IS NULL` 统一返回 404。

### 4.15 功能依赖小结

```text
imports ──► books / chapters / segments / jobs / files / characters
tts     ──► segments / voices / jobs / files / common::storage / provider
audio   ──► chapters / segments / books / jobs / files / common::ffmpeg
exports ──► chapters / books / jobs / files / common::storage
books   ──► chapters / segments / jobs / files
chapters──► segments（重切段）
characters ─► segments / jobs
voices  ──► tts::provider / files
files   ──► common::storage
recycle_bin ─► books / files / common::storage / jobs
jobs    ──► 无（被其他功能调用）
```

禁止反向依赖；同一功能内文件可自由调用。

## 5. Worker 设计

### 5.1 Dispatcher

```rust
pub struct Dispatcher {
    state: Arc<WorkerState>,
    semaphore: Arc<Semaphore>, // WORKER_CONCURRENCY
}
```

主循环：

1. 每 30s 调 `jobs::queue::reclaim_stale`。
2. 获取信号量 → `claim_next(worker_id)`。
3. 有任务：`spawn(handle_job(job))`；无任务：sleep 1s。
4. 收到停机信号：停止认领，等待运行任务最多 30s。

`handle_job`：

1. 创建 `CancellationToken`。
2. 启动心跳任务（10s），心跳失败 → 取消任务。
3. 按 `job_type` 分发：

| 任务类型 | Worker 文件 | 调用 |
| --- | --- | --- |
| `import_parse` | `worker/jobs/import_parse.rs` | `features::imports::service::run_import` |
| `character_detect` | `worker/jobs/character_detect.rs` | `features::characters::service::run_detect` |
| `chapter_tts` | `worker/jobs/chapter_tts.rs` | `features::tts::service::run_tts` |
| `chapter_merge` | `worker/jobs/chapter_merge.rs` | `features::audio::service::run_merge` |
| `book_export` | `worker/jobs/book_export.rs` | `features::exports::service::run_export` |

4. 记录 `JobOutcome` → `jobs::service::complete`。
5. 停止心跳、清理临时目录、记录耗时。

### 5.2 ProgressReporter

- 维护 `done/failed/skipped/total` 原子计数。
- 每 500ms 最多写一次数据库。
- 写入 `progress`、`current_step`、`updated_at`。
- 任务结束强制 flush。

### 5.3 任务检查点

每个 Handler 在每个子项开始前检查：

1. `ctx.cancel.is_cancelled()`；
2. 数据库 `jobs.cancel_requested`（每 20 个子项或 5s 查一次）。

发现取消：停止领取新子项，未开始子项标 `skipped`，任务 `canceled`。

## 6. 共享能力 `common/`

### 6.1 `db.rs`

- `connect(config)`：创建连接池，重试 30 次（间隔 2s）。
- `migrate(pool)`：`sqlx::migrate!` 执行迁移。
- `ping(pool)`：健康检查。

### 6.2 `storage.rs`

```rust
pub struct Storage {
    backend: StorageBackend,
}

enum StorageBackend {
    RustFs(RustFsClient),
    Memory(Mutex<HashMap<String, Bytes>>),
}
```

方法：`put/get/delete/head/ensure_bucket`。

- RustFS 使用 `aws-sdk-s3`，`force_path_style(true)`。
- `Storage::memory()` 供测试使用。
- 不使用 trait/端口抽象；通过枚举切换后端。

### 6.3 `ffmpeg.rs`

```rust
pub struct Ffmpeg {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
    semaphore: Arc<Semaphore>,
}
```

方法：

| 方法 | 说明 |
| --- | --- |
| `probe(path)` | 返回时长、采样率、声道 |
| `generate_silence(path, ms, format)` | 生成静音 MP3 |
| `merge_with_loudnorm(input)` | 拼接 + 响度 + ID3 |
| `version()` | `/readyz` 与启动自检 |

约束：

1. 只通过参数数组调用 `tokio::process::Command`，禁止 shell 拼接。
2. `merge` 超时 600s，超时杀进程。
3. FFmpeg 并发由 `FFMPEG_CONCURRENCY` 控制。

### 6.4 `text.rs` / `encoding.rs`

- `text.rs`：`normalize`、`split_chapters`、`split_segments`、`classify_paragraph`、`sanitize_filename`。
- `encoding.rs`：BOM → 严格 UTF-8 → GB18030；替换字符比例检查。
- 均为纯函数，无 I/O、无数据库。

### 6.5 其他

| 文件 | 内容 |
| --- | --- |
| `hash.rs` | `sha256_hex`、`cache_key` |
| `id.rs` | 请求 ID（UUID v7）、对象 Key、worker_id |
| `pagination.rs` | `Page<T>`、分页参数 |
| `enums.rs` | BookStatus / ChapterStatus / SegmentStatus / JobType / JobStatus 等 |
| `time.rs` | UTC 时间与格式化 |

## 7. 事务、并发与错误规范

| 规范 | 内容 |
| --- | --- |
| 事务开启 | 由发起业务动作的功能 service 开启 |
| 跨功能写 | 传 `&mut Transaction` 给归属功能 service |
| 长耗时操作 | 下载/合成/FFmpeg 不持有事务；完成后短事务落库 |
| 并发控制 | 段落状态 CAS；任务 `SKIP LOCKED`；Provider/FFmpeg 信号量 |
| 幂等 | 已完成段落跳过；对象 Key 确定性；任务重试只处理失败项 |
| 超时 | Provider 45s；ffprobe 15s；FFmpeg 600s；HTTP 30s |
| 重试 | Provider/上传 2–3 次；业务校验失败不重试 |
| 错误 | 统一 `AppError` → 错误码；5xx 记录完整错误链 |
| 日志 | `request_id` / `job_id` 贯穿；禁止密码、JWT、密钥、完整文本 |
| 清理 | `TempDirGuard`；启动时清理 24h 前临时目录 |

## 8. 功能验收点

1. `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 全绿。
2. 新增一个功能只需新增 `features/<name>/` 并在两处注册（`features/mod.rs`、`router.rs`）。
3. 任意功能目录内可以完成「路由 → 参数 → 业务 → SQL → 模型」的阅读与修改，不需要跨层跳转。
4. 跨功能调用只出现在 `service.rs`，`repo.rs` 不被其他功能引用。
5. 日志中可通过 `request_id` / `job_id` 串联一次完整请求或任务。
6. 测试使用测试数据库、内存 Storage、Mock TTS Provider，不依赖真实 TTS 接口。
