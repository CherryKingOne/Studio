# NovelCast 功能优先目录结构树（待审核稿）

> 文档编号：09  
> 版本：v0.1  
> 状态：**仅用于目录结构评审，尚未批准开发**  
> 适用范围：当前后端项目 `backend/`  
> 重要约束：在你明确回复“审核通过并按此结构开发/修改代码”之前，本目录树不会被用于迁移、重命名、删除或新增业务代码。

## 1. 这份文档要解决的问题

当前项目已经可以运行，但目录形态仍然带有明显的固定技术层模板：每个功能目录重复放置 `routes.rs`、`handler.rs`、`service.rs`、`repo.rs`、`model.rs`、`dto.rs`。这种方式虽然名称上叫“按功能”，实际查找代码时仍然要先按技术类型跳转，不能直接从用户要完成的事情找到代码。

本稿提出一次完整的目录结构调整，目标是让目录先表达产品功能和用户操作，再表达实现细节：

- 看到目录名就能知道它负责什么产品能力。
- 一个用户操作尽量在一个文件或一个小目录内闭环，包括接口、参数、业务步骤和数据读写。
- 只有确实被多个操作复用的内容才提取为同一功能下的共享文件。
- 不强制所有功能都拥有相同的文件集合。
- 不新增 `domain/`、`application/`、`infrastructure/`、`ports/`、`adapters/`、聚合根、仓储接口等 DDD 或伪 DDD 结构。
- 不把技术名词重新包装成另一套分层目录；目录的第一层级必须是产品功能。

## 2. 评审边界

### 2.1 当前只评审目录，不进行开发

本次交付只有这份 Markdown 文档。当前不会执行以下操作：

- 不移动、重命名或删除现有 Rust 文件。
- 不新建目标目录对应的代码文件。
- 不修改路由、数据库、API、配置或业务逻辑。
- 不把旧文档中的“已进入开发”当作本次授权。

待你审核目录树后，如果你明确给出开发指令，再单独制定迁移顺序和代码修改清单。

### 2.2 现状基线（以当前文件为准）

当前实际存在的后端功能目录如下：

```text
backend/src/
├── app.rs
├── bin/worker.rs
├── common/
├── config.rs
├── error.rs
├── features/
│   ├── audio/
│   ├── auth/
│   ├── books/
│   ├── chapters/
│   ├── characters/
│   ├── exports/
│   ├── files/
│   ├── health/
│   ├── imports/
│   ├── jobs/
│   ├── segments/
│   ├── tts/
│   └── voices/
├── lib.rs
├── middleware/
├── router.rs
├── state.rs
└── worker/
    ├── dispatcher.rs
    └── mod.rs
```

当前确实存在的共享能力包括数据库、对象存储、FFmpeg、文本处理、临时文件、分页、时间、日志、信号和通用响应等。旧目录规划中提到的 `openapi.rs`、`worker/jobs/`、`worker/progress.rs`、`assets/`、`fixtures/`、测试目录等不作为当前已存在内容处理。

## 3. 新结构的组织规则

### 3.1 第一优先级：产品功能

目标树不再使用一个笼统的 `features/` 容器来隐藏产品能力，而是把产品功能直接放在 `src/` 下。建议的一级功能为：

| 一级目录 | 面向用户的能力 |
| --- | --- |
| `account/` | 登录、当前账号、修改密码 |
| `library/` | 书籍、章节、段落及书籍统计 |
| `text_import/` | 上传文本、识别编码、切章、切段、导入执行 |
| `casting/` | 角色识别、角色管理、音色目录、试听和音色分配 |
| `speech/` | 单段/整章语音合成、合成缓存、TTS 供应商 |
| `audio/` | 章节拼接、段落/章节音频访问、音频失效 |
| `delivery/` | 整书导出、导出状态、文件下载 |
| `task_center/` | 任务列表、详情、取消、重试、进度事件和队列 |
| `system/` | 健康检查等系统可见能力 |

### 3.2 第二优先级：用户操作或独立能力

功能目录内部按实际操作命名，例如 `login.rs`、`book_cover.rs`、`split_chapters.rs`、`merge_chapter.rs`。不要求每个目录都出现同名的 `handler`、`service`、`repo` 或 `dto` 文件。

一个操作文件可以同时包含：

- HTTP 路由注册或该操作的路由入口。
- 请求参数读取和输入校验。
- 该操作的业务步骤。
- 该操作所需的数据查询和写入。
- 响应类型和错误转换。

只有当内容明显被多个操作共同使用时，才在当前功能目录中增加 `shared.rs`、`types.rs`、`data.rs` 等文件；这些文件必须说明共享范围，不得为了形式完整而创建。

### 3.3 跨功能调用

- 一个功能需要另一个功能时，调用对方公开的功能函数，不直接依赖对方的内部文件。
- 不为跨功能调用创建一套接口、端口或仓储抽象。
- 数据表的读写跟随实际业务功能放置；不单独建立“数据访问层”目录。
- `shared/` 只放与产品业务无关的技术能力，不能把书籍、章节、角色、任务等业务概念塞进去。
- `runtime/` 只负责启动、路由汇总、状态和全局中间件，不承载具体产品流程。
- `worker/` 只负责领取任务和触发对应功能流程，不复制业务规则。

### 3.4 命名规则

- 目录和文件使用小写英文和下划线，例如 `text_import/`、`book_cover.rs`。
- 文件名表达“用户动作”或“产品能力”，避免 `layer`、`manager`、`processor`、`repository` 等空泛命名。
- 供应商名称只出现在供应商功能目录下，例如 `speech/providers/openai.rs`。
- 共享技术文件使用具体名词，例如 `object_storage.rs`、`temporary_files.rs`，不使用 `misc.rs`、`utils.rs`。

## 4. 目标根目录树（待审核）

以下是审核通过后才允许实施的目标树。树中的文件是规划位置，不代表当前已经存在。

```text
/Users/leo/Documents/One-project/Studio/
├── README.md
├── docs/
│   ├── 00-overview.md
│   ├── 01-directory-structure.md
│   ├── 02-architecture.md
│   ├── 03-database-design.md
│   ├── 04-api-design.md
│   ├── 05-core-workflows.md
│   ├── 06-backend-modules.md
│   ├── 07-roadmap-testing.md
│   ├── 08-review-checklist.md
│   └── 09-functional-directory-tree-review.md  # 本次评审稿
└── backend/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── rust-toolchain.toml
    ├── .env.example
    ├── .dockerignore
    ├── .gitignore
    ├── Dockerfile
    ├── docker-compose.yml
    ├── README.md
    ├── scripts/
    │   └── check.sh
    └── src/
        ├── lib.rs
        ├── main.rs
        ├── bin/
        │   └── worker.rs
        │
        ├── runtime/                         # 启动与 HTTP 组装，不放产品业务
        │   ├── mod.rs
        │   ├── server.rs                     # 创建 API 应用并启动监听
        │   ├── router.rs                     # 汇总各功能公开路由
        │   ├── state.rs                      # API/Worker 运行状态
        │   ├── config.rs                     # 环境变量与运行配置
        │   ├── error.rs                      # 全局错误响应格式
        │   └── middleware/
        │       ├── mod.rs
        │       ├── request_id.rs             # 请求 ID
        │       ├── auth_guard.rs             # Bearer Token 校验入口
        │       └── rate_limit.rs             # 登录、试听等限流
        │
        ├── account/                          # 账号与访问
        │   ├── mod.rs
        │   ├── login.rs                      # 登录、密码校验、Token 签发
        │   ├── current_user.rs               # 当前账号信息
        │   ├── change_password.rs            # 修改密码
        │   ├── session.rs                    # Token 解析和账号身份数据
        │   └── account_data.rs               # 账号相关数据操作（仅在确有复用时保留）
        │
        ├── library/                          # 书籍内容工作区
        │   ├── mod.rs
        │   ├── books/
        │   │   ├── mod.rs
        │   │   ├── browse.rs                 # 书籍列表、详情
        │   │   ├── create.rs                 # 创建书籍
        │   │   ├── edit.rs                   # 修改、删除书籍
        │   │   ├── cover.rs                  # 封面上传与关联
        │   │   └── statistics.rs            # 书籍统计
        │   ├── chapters/
        │   │   ├── mod.rs
        │   │   ├── browse.rs                 # 章节列表、详情
        │   │   ├── edit.rs                   # 章节正文和标题编辑
        │   │   └── resegment.rs              # 重新切段入口
        │   └── segments/
        │       ├── mod.rs
        │       ├── browse.rs                 # 段落列表
        │       ├── edit.rs                   # 单段编辑
        │       └── batch_edit.rs             # 批量编辑
        │
        ├── text_import/                      # 文本导入与解析
        │   ├── mod.rs
        │   ├── upload.rs                     # 上传 TXT、建立导入任务
        │   ├── decode.rs                     # UTF-8/GBK/GB18030 识别与解码
        │   ├── split_chapters.rs             # 章节识别与切分
        │   ├── split_segments.rs             # 段落切分和旁白/对白分类
        │   ├── execute.rs                    # Worker 执行完整导入流程
        │   └── parsed_book.rs                # 解析结果类型（被多个导入步骤复用）
        │
        ├── casting/                          # 角色与音色配置
        │   ├── mod.rs
        │   ├── characters/
        │   │   ├── mod.rs
        │   │   ├── manage.rs                 # 角色增删改查
        │   │   └── detect.rs                 # 说话人候选和角色识别
        │   ├── voices/
        │   │   ├── mod.rs
        │   │   ├── catalog.rs                # 音色列表和手动登记
        │   │   ├── sync.rs                   # 从供应商同步音色
        │   │   ├── preview.rs                # 音色试听
        │   │   └── assignment.rs             # 旁白、角色、段落音色解析
        │   └── cast_data.rs                  # 角色/音色之间确有复用时才保留
        │
        ├── speech/                           # TTS 语音合成
        │   ├── mod.rs
        │   ├── synthesize_chapter.rs         # 整章合成任务
        │   ├── synthesize_segment.rs         # 单段合成任务
        │   ├── cache.rs                      # 合成缓存键与缓存读写
        │   └── providers/
        │       ├── mod.rs
        │       ├── openai.rs
        │       ├── minimax.rs
        │       ├── stepfun.rs
        │       └── mock.rs                   # 自动化测试用供应商（是否纳入由审核决定）
        │
        ├── audio/                            # 音频处理与试听下载
        │   ├── mod.rs
        │   ├── merge_chapter.rs              # 段落拼接、停顿、响度和编码
        │   ├── chapter_audio.rs              # 章节音频读取、删除和失效
        │   ├── segment_audio.rs              # 段落音频读取
        │   └── invalidate.rs                 # 文本/音色修改后的音频失效规则
        │
        ├── delivery/                         # 产物导出与文件交付
        │   ├── mod.rs
        │   ├── create_book_export.rs         # 创建整书导出
        │   ├── run_book_export.rs            # 组装 ZIP、manifest 和封面
        │   ├── export_status.rs              # 查询导出状态
        │   ├── download_export.rs            # 下载导出的 ZIP
        │   └── download_file.rs              # 通用文件下载
        │
        ├── task_center/                      # 任务管理和队列
        │   ├── mod.rs
        │   ├── task_list.rs                  # 任务列表
        │   ├── task_detail.rs                # 任务详情与子项
        │   ├── cancel_task.rs                # 取消任务
        │   ├── retry_task.rs                 # 重试任务
        │   ├── task_events.rs                # SSE 进度事件
        │   ├── queue.rs                      # 入队、认领、心跳、回收、完成
        │   └── task_data.rs                  # 任务数据读写和状态类型
        │
        ├── system/                           # 系统可见功能
        │   ├── mod.rs
        │   └── health.rs                     # 首页、存活检查、就绪检查
        │
        ├── worker/                           # Worker 进程入口与任务触发
        │   ├── mod.rs
        │   ├── dispatcher.rs                 # 并发、停机、领取任务
        │   └── tasks/
        │       ├── mod.rs
        │       ├── import.rs                 # 调用 text_import::execute
        │       ├── character_detection.rs   # 调用 casting::characters::detect
        │       ├── speech.rs                 # 调用 speech 合成流程
        │       ├── audio_merge.rs            # 调用 audio::merge_chapter
        │       └── book_export.rs            # 调用 delivery::run_book_export
        │
        └── shared/                           # 无业务含义的技术共享能力
            ├── mod.rs
            ├── database.rs                   # MySQL 连接、建表检查
            ├── object_storage.rs             # RustFS/S3 对象读写
            ├── ffmpeg.rs                     # ffmpeg/ffprobe 调用
            ├── text_cleaning.rs              # 通用文本清理，不放导入流程
            ├── hashing.rs                    # SHA-256 等通用哈希
            ├── pagination.rs                 # 通用分页计算
            ├── time.rs                       # UTC 时间工具
            ├── logging.rs                    # 日志初始化
            ├── shutdown.rs                   # 优雅停机信号
            └── temporary_files.rs            # 临时文件生命周期
```

## 5. 当前目录到目标目录的功能映射

这张表只用于评审迁移方向，不代表现在已经执行了文件移动。

| 当前路径 | 目标功能路径 | 调整重点 |
| --- | --- | --- |
| `features/auth/` | `account/` | 按登录、当前账号、改密拆成用户操作 |
| `features/books/` | `library/books/` | 按浏览、创建、编辑、封面、统计拆分 |
| `features/chapters/` | `library/chapters/` | 按浏览、编辑、重新切段组织 |
| `features/segments/` | `library/segments/` | 按浏览、单段编辑、批量编辑组织 |
| `features/imports/` | `text_import/` | 上传、解码、切章、切段、执行各自归位 |
| `features/characters/` | `casting/characters/` | 管理和识别分开表达 |
| `features/voices/` | `casting/voices/` | 目录、同步、试听、分配分开表达 |
| `features/tts/` | `speech/` | 合成动作、缓存、供应商各归到语音生产能力 |
| `features/audio/` | `audio/` | 拼接、章节音频、段落音频、失效分开表达 |
| `features/exports/` | `delivery/` | 创建、执行、状态、下载按交付动作组织 |
| `features/files/` | `delivery/download_file.rs` | 文件下载是交付能力的一部分 |
| `features/jobs/` | `task_center/` | 任务管理接口和队列集中在任务中心 |
| `features/health/` | `system/health.rs` | 健康检查作为系统可见功能保留 |
| `common/` | `shared/` | 只保留数据库、存储、FFmpeg、日志等技术能力 |
| `app.rs`、`router.rs`、`state.rs`、`config.rs`、`error.rs` | `runtime/` | 统一放启动与 HTTP 组装代码 |
| `middleware/` | `runtime/middleware/` | 全局请求处理能力归运行时 |
| `worker/dispatcher.rs` | `worker/dispatcher.rs` | 保留调度职责，按任务类型拆到 `worker/tasks/` |

## 6. 路由归属建议

路由仍然按产品能力归属，不按 HTTP 方法或技术层拆分：

| 产品能力 | 路由范围 | 目标位置 |
| --- | --- | --- |
| 系统 | `/`、`/healthz`、`/readyz` | `system/health.rs` |
| 账号 | `/api/v1/auth/*` | `account/login.rs`、`account/current_user.rs`、`account/change_password.rs` |
| 书籍 | `/api/v1/books`、`/api/v1/books/{id}` 及封面/统计 | `library/books/*` |
| 导入 | `/api/v1/books/import` | `text_import/upload.rs` |
| 章节 | `/api/v1/books/{id}/chapters`、`/api/v1/chapters/{id}` | `library/chapters/*` |
| 段落 | `/api/v1/chapters/{id}/segments`、`/api/v1/segments/*` | `library/segments/*` |
| 角色 | `/api/v1/books/{id}/characters`、`/api/v1/characters/*` | `casting/characters/*` |
| 音色 | `/api/v1/voices/*` | `casting/voices/*` |
| 合成 | `/api/v1/chapters/{id}/tts`、`/api/v1/segments/{id}/tts` | `speech/synthesize_*.rs` |
| 音频 | `/api/v1/chapters/{id}/merge`、章节/段落音频 | `audio/*` |
| 导出 | `/api/v1/books/{id}/exports`、`/api/v1/exports/*` | `delivery/*` |
| 任务 | `/api/v1/jobs/*` | `task_center/*` |
| 文件 | `/api/v1/files/{id}` | `delivery/download_file.rs` |

## 7. 功能依赖方向（目标约束）

```text
runtime
  └── 只负责启动、路由汇总、全局状态和中间件

account ───────────────┐
library ────────────────┤
text_import ────────────┤
casting ────────────────┤──► shared（数据库、存储、FFmpeg、日志等）
speech ─────────────────┤
audio ──────────────────┤
delivery ───────────────┤
task_center ────────────┘

worker ──► task_center 领取任务
worker ──► 调用 text_import / casting / speech / audio / delivery 的完整功能流程
```

约束说明：

1. `runtime` 不调用任何具体业务数据操作，只负责组装。
2. `shared` 不依赖 `account`、`library`、`speech` 等产品功能。
3. `worker` 不写导入、合成、拼接或导出规则；规则留在对应功能目录。
4. `task_center` 管任务生命周期；业务功能只负责创建自己需要的任务并提供执行入口。
5. `speech/providers/` 只解决供应商请求差异，不向其他功能暴露供应商细节。
6. 跨功能调用使用清晰的公开函数；不新增 Repository trait、端口或适配器层。

## 8. 不在本次目标树中默认新增的内容

以下内容只有在你单独确认后才进入后续规划：

- `frontend/` 前端工程。
- `tests/` 集成测试目录及测试夹具目录。
- `assets/` 静态资源目录。
- `openapi.rs` 或独立 OpenAPI 生成目录。
- 独立数据库迁移文件目录。
- Redis、消息队列或其他新的基础设施目录。
- 多用户、权限、租户、计费等产品功能目录。

## 9. 审核后才开始的迁移顺序（仅供评审）

如果你审核通过，建议按下面顺序执行，实际开始仍需你的明确指令：

1. 先冻结一级功能名称和路由归属。
2. 建立目标模块的空壳与模块声明，确保 Cargo 仍可编译。
3. 迁移 `account`、`library`、`system` 等同步功能。
4. 迁移 `text_import`、`casting`、`speech`、`audio` 等生产流程。
5. 迁移 `task_center` 和 `worker/tasks`，核对每一种任务类型。
6. 逐步删除旧的 `features/`、`common/`、旧运行时文件，删除前逐项确认引用已清理。
7. 执行格式检查、静态检查、测试和核心 API 验收。

上述顺序只是候选执行方案，不构成当前开发授权。

## 10. 本次请你审核的内容

请重点确认以下项目：

1. 是否同意取消 `src/features/` 这一层，让产品功能直接位于 `src/` 下。
2. 是否同意采用 `account / library / text_import / casting / speech / audio / delivery / task_center` 这组功能名称。
3. 是否同意按用户操作拆文件，而不是为每个目录固定生成 `routes/handler/service/repo/model/dto`。
4. 是否同意将 `common/` 改为只表达技术共享能力的 `shared/`。
5. 是否同意把任务管理集中到 `task_center/`，把任务执行入口放到 `worker/tasks/`。
6. 是否有需要改名、合并、拆分或新增的功能目录。

请直接指出目录、文件名或规则的修改意见。只有在你明确说“审核通过并开始开发/修改代码”后，才进入下一阶段。 
