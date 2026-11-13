# 07 开发计划与测试验收

> 文档版本：v0.2（评审稿）
> 本文定义开发阶段（M0–M6）、每阶段任务清单与验收门槛、测试方案、风险登记。
> **本版变更：任务产出物路径改为按功能模块组织（`features/*` + `common/*`）。**
> **不包含工期估算**；阶段按依赖顺序推进，每阶段通过验收门槛后再进入下一阶段。

## 1. 阶段总览

| 阶段 | 名称 | 目标 | 退出条件 |
| --- | --- | --- | --- |
| M0 | 工程基建 | 可构建、可启动、可连接依赖 | `docker compose up` 后 `/readyz` 全绿 |
| M1 | 认证与书籍基础 | 登录、书籍 CRUD、文件存储 | 封面可上传下载，鉴权生效 |
| M2 | 导入解析 | TXT → 章节 → 段落 | UTF-8/GBK 样例书导入成功，SSE 可见进度 |
| M3 | 角色与音色 | 角色识别、音色目录、音色分配 | 样例书角色可识别、可试听、可绑定 |
| M4 | TTS 生产 | 批量合成、缓存、重试、取消、进度 | 整章合成成功，缓存命中，失败可重试 |
| M5 | 拼接与导出 | 章节 MP3、响度、ID3、ZIP | 样例书可导出 ZIP 并播放 |
| M5+ | 回收站 | 软删除闭环、恢复、彻底删除、保留期自动清理 | 回收站接口与自动清理实测通过 |
| M6 | 加固与验收 | 测试补全、安全、文档、部署 | `check.sh` 全绿 + 端到端验收通过 |

## 2. 阶段任务与验收

### 2.1 M0 工程基建

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M0-01 | 创建 `backend/` Cargo 工程，两个 bin（`novelcast-server`、`novelcast-worker`） | `Cargo.toml`、`main.rs`、`bin/worker.rs`、`lib.rs` |
| M0-02 | 配置加载与校验（全部来自 `.env`） | `config.rs`、`.env.example` |
| M0-03 | 错误类型与统一响应包络 | `error.rs`、`common/pagination.rs` |
| M0-04 | 请求 ID、Trace、CORS、BodyLimit 中间件 | `middleware/*` |
| M0-05 | 日志初始化（pretty/json） | `main.rs`、`bin/worker.rs` |
| M0-06 | MySQL 连接池 + 代码内嵌建表（`schema.rs`） | `shared/db.rs`、`shared/schema.rs` |
| M0-07 | RustFS 客户端 + `ensure_bucket` | `common/storage.rs` |
| M0-08 | 健康检查 `/healthz`、`/readyz` | `features/health/*` |
| M0-09 | Dockerfile 多阶段构建 + Compose 四服务 | `Dockerfile`、`docker-compose.yml` |
| M0-10 | `scripts/check.sh`（fmt/clippy/test） | `scripts/check.sh` |

M0 验收：

1. `cargo build` 成功，`cargo clippy -- -D warnings` 无告警。
2. `docker compose up -d` 后 4 个服务健康。
3. `GET /healthz` 返回 200；`GET /readyz` 中 mysql/rustfs/ffmpeg 均为 ok。
4. 重启容器后数据卷保留，迁移不重复报错。

### 2.2 M1 认证与书籍基础

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M1-01 | 启动引导创建管理员（users 为空时） | `features/auth/service.rs` 的 bootstrap |
| M1-02 | 登录、JWT 签发与校验、修改密码 | `features/auth/*`、`middleware/auth.rs` |
| M1-03 | 登录限流 | `middleware/rate_limit.rs` |
| M1-04 | 书籍 CRUD + 软删除 + 默认 settings | `features/books/*` |
| M1-05 | 文件上传（封面）与鉴权下载 | `common/storage.rs`、`features/files/*`、`features/books/service.rs` |
| M1-06 | 书籍统计接口 | `features/books/service.rs::stats` |
| M1-07 | 分页与统一错误码 | `common/pagination.rs`、`error.rs` |
| M1-08 | 单元测试：密码哈希、JWT、分页参数 | `tests/api_auth.rs` |

M1 验收：

1. 正确密码登录返回 Token；错误密码返回 `10003`；超限返回 `10004`。
2. 携带 Token 可 CRUD 书籍；不带 Token 返回 `401`。
3. 上传 5MB 内 jpg 成功并可下载；6MB 文件返回 `413`；非图片返回 `415`。
4. 软删除后列表不返回，详情返回 `404`。
5. 日志中不出现密码与完整 Token。

### 2.3 M2 导入解析

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M2-01 | 编码检测与解码 | `common/encoding.rs` + 单测 |
| M2-02 | 文本规范化、切章（主正则/特殊标题/兜底） | `common/text.rs::split_chapters` + 单测 |
| M2-03 | 段落切分、引号分类、碎段合并 | `common/text.rs::split_segments` + 单测 |
| M2-04 | 导入 API：上传 → files → book → job | `features/imports/*` |
| M2-05 | Worker `import_parse` Handler | `worker/jobs/import_parse.rs` |
| M2-06 | 章节/段落列表与详情接口 | `features/chapters/*`、`features/segments/*` |
| M2-07 | 章节编辑、重新切段、失效逻辑 | 对应功能 `service.rs` + `routes.rs` |
| M2-08 | 任务查询与 SSE | `features/jobs/*` |
| M2-09 | 集成测试：UTF-8/GBK 导入 | `tests/import_flow.rs` |

M2 验收：

1. 两份样例小说导入后：章节数 ≥3、正文无标题残留、段落数 >0。
2. GBK 文件无乱码；替换字符比例检查生效。
3. 导入任务 SSE 依次出现 `切分章节`、`切分段落`、`角色识别` 步骤（角色识别在 M3 前可为占位）。
4. 修改章节正文在有音频时被拒绝；无音频时成功并可重新切段。
5. 段落编辑后状态为 `pending`，章节音频为空。

### 2.4 M3 角色与音色

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M3-01 | 角色 CRUD + 唯一约束 + 删除失效 | `features/characters/*` |
| M3-02 | 音色手动登记（OpenAI/MiniMax/阶跃）+ 列表 | `features/voices/service.rs`、`features/tts/provider/{openai,minimax}.rs` |
| M3-03 | 音色试听（同步合成） | `features/voices/service.rs::preview` |
| M3-04 | 规则角色识别 + 候选池 + 置信度 | `features/characters/detector.rs` |
| M3-05 | `character_detect` Worker Handler | `worker/jobs/character_detect.rs` |
| M3-06 | 音色解析优先级 | `features/voices/service.rs::resolve_voice` |
| M3-07 | 角色/音色变更的失效逻辑 | `features/characters/service.rs`、`features/books/service.rs` |
| M3-08 | 测试：识别规则、优先级、失效 | 单测 + `tests/import_flow.rs` 扩展 |

M3 验收：

1. 样例书中至少识别出 2 个角色；重跑不产生重名角色。
2. 无法判断的对白 `speaker_id` 为空且 `speaker_confidence` 有值。
3. 音色同步返回数量 >0；试听返回可播放 MP3。
4. 段落列表返回正确的 `resolved_voice`（含来源）。
5. 修改角色音色后，只有受影响段落变 `pending`。

### 2.5 M4 TTS 生产

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M4-01 | MySQL 队列实现（认领/心跳/回收/完成） | `features/jobs/queue.rs` |
| M4-02 | Worker Dispatcher + Handler 分发 + 优雅停机 | `worker/dispatcher.rs` |
| M4-03 | `TtsProvider` trait + Registry + Mock | `features/tts/provider/*` |
| M4-04 | OpenAI / MiniMax / 阶跃星辰 Provider | `features/tts/provider/{openai,minimax}.rs` |
| M4-05 | 缓存键、缓存写入/命中 | `features/tts/{service,cache,repo}.rs` |
| M4-06 | 整章合成任务 + 段落并发 + 重试 | `features/tts/service.rs`、`worker/jobs/chapter_tts.rs` |
| M4-07 | 单段合成任务 | `features/tts/{routes,handler,service}.rs` |
| M4-08 | 取消与重试 | `features/jobs/service.rs`、Dispatcher |
| M4-09 | 测试：Mock 合成、缓存、失败重试、取消 | `tests/tts_flow.rs`、`tests/job_queue.rs` |

M4 验收：

1. Mock Provider 下整章 10 段全部 `done`，任务 `succeeded`。
2. 相同参数再次合成命中缓存（Provider 调用次数为 0）。
3. 强制 `force=true` 绕过缓存。
4. Mock 连续失败 3 次后段落 `failed`，任务 `partial`；其他段落不受影响。
5. 取消后不再发起新的 Provider 调用。
6. 杀掉 Worker 后任务在 90s 内被回收并继续。
7. SSE 进度单调递增，终态事件正确。

### 2.6 M5 拼接与导出

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M5-01 | ffprobe 封装与音频校验 | `common/ffmpeg.rs::probe` |
| M5-02 | FFmpeg 拼接 + loudnorm + ID3 | `common/ffmpeg.rs::merge_with_loudnorm` |
| M5-03 | 静音片段生成与 concat 清单 | `worker/jobs/chapter_merge.rs` |
| M5-04 | 章节音频上传与事务更新 | `features/audio/service.rs`、`worker/jobs/chapter_merge.rs` |
| M5-05 | 章节音频下载/失效接口 | `features/audio/routes.rs` |
| M5-06 | 整书 ZIP 导出（manifest/封面/命名） | `features/exports/service.rs`、`worker/jobs/book_export.rs` |
| M5-07 | 导出查询与下载 | `features/exports/routes.rs` |
| M5-08 | 测试：fixture 音频拼接、ZIP 结构 | `tests/merge_export_flow.rs` |

M5 验收：

1. 用 fixture 音频拼接，输出时长 ≈ 输入 + 停顿，误差 <5%。
2. 响度在目标 ±1 LUFS 内（`ffmpeg loudnorm` 测量）。
3. ID3 的 title/artist/album/track 正确。
4. 拼接失败不覆盖旧章节音频。
5. 导出 ZIP 包含正确数量的 MP3、`manifest.json`、可选封面。
6. ZIP 文件名有序、无非法字符。

### 2.7 M5+ 回收站（增量）

> 详细设计见 [10 回收站设计](10-recycle-bin-design.md)。

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M5P-01 | `books.deleted_by` 迁移与删除人记录 | `schema.rs`、`book_data.rs` |
| M5P-02 | 回收站列表 / 汇总 / 恢复 / 彻底删除 / 批量 / 清空 | `src/recycle_bin/*` |
| M5P-03 | 回收站期间章节/段落/角色访问隔离 | `chapter_data.rs`、`segment_data.rs`、`characters/data.rs` |
| M5P-04 | Worker 保留期自动清理 | `worker/dispatcher.rs`、`RECYCLE_BIN_RETENTION_DAYS` |
| M5P-05 | 接口文档与设计文档 | `docs/10-recycle-bin-design.md`、`docs/api/recycle-bin/*` |

M5+ 验收：

1. 删除书籍后在回收站可见，详情与子资源接口均为 404。
2. 恢复后书籍、章节、段落、角色重新可见。
3. 书名冲突时 `on_conflict=rename` 自动改名成功。
4. 彻底删除后书籍、章节、段落、任务与未共享文件被清理。
5. 保留期到期后 Worker 自动清理。

### 2.8 M6 加固与验收

| 任务ID | 任务 | 产出 |
| --- | --- | --- |
| M6-01 | 补齐集成测试与边界用例 | `tests/*` |
| M6-02 | 安全复查（CORS、上传、日志脱敏、限流） | 检查记录 |
| M6-03 | OpenAPI 完整性与 `/docs` 开关 | `openapi.rs` |
| M6-04 | 生产 Compose 配置与重启策略 | `docker-compose.yml`、`production.toml` |
| M6-05 | 端到端验收脚本与演示数据 | `scripts/`、`fixtures/` |
| M6-06 | 文档与实现一致性核对 | 文档修订记录 |

M6 验收：

1. `scripts/check.sh` 全绿。
2. 端到端脚本完成：导入 → 识别 → 音色 → 合成（Mock）→ 拼接 → 导出。
3. 生产配置下 CORS、日志格式、`/docs` 关闭符合预期。
4. 文档与接口实现无偏差（抽查 10 个接口）。

## 3. 测试方案

### 3.1 测试分层

| 层级 | 范围 | 工具 | 依赖 |
| --- | --- | --- | --- |
| 单元测试 | 纯函数、状态机、参数校验、缓存键 | `#[test]` / `tokio::test` | 无外部依赖 |
| 数据访问测试 | SQL 正确性、索引、事务 | `#[tokio::test]` + 测试 MySQL | MySQL |
| API 集成测试 | 路由、鉴权、DTO、错误码 | `tower::ServiceExt::oneshot` | 测试 MySQL + 内存 Storage + Mock TTS |
| 流程测试 | 导入/合成/拼接/导出 | 测试 App + Worker 手动驱动 | MySQL + 内存 Storage + Mock TTS + fixture 音频 |
| 手工验收 | 真实 OpenAI/MiniMax/阶跃 TTS、真实播放 | `curl` / 播放器 | 外网 + API Key |

### 3.2 测试替身

| 能力 | 生产实现 | 测试实现 |
| --- | --- | --- |
| 对象存储 | RustFS（`common/storage.rs`） | `common/storage.rs` 的内存后端 |
| TTS 引擎 | OpenAI / MiniMax / 阶跃星辰（`features/tts/provider/`） | Mock Provider（后续补） |
| 音频处理 | FFmpeg（`common/ffmpeg.rs`） | 单元测试用真实 ffprobe；拼接用 fixture |
| 任务队列 | MySQL（`features/jobs/queue.rs`） | 使用测试 MySQL，不额外 Mock |

### 3.3 必备用例清单

| 用例ID | 场景 | 预期 |
| --- | --- | --- |
| UT-01 | UTF-8 / UTF-8 BOM / GBK 解码 | 文本一致，编码类型正确 |
| UT-02 | 乱码文件 | 替换字符比例检查触发错误 |
| UT-03 | 章节标题正则（第X章/回/节/卷/序章） | 切分正确，标题残留为 0 |
| UT-04 | 无章节标题兜底 | 按长度切分，标题为「第 N 部分」 |
| UT-05 | 引号分类（整段对白/混合/无引号） | `dialogue/narration` 正确 |
| UT-06 | 超长段落按句末切分 | 每段 ≤max_chars，标点不丢 |
| UT-07 | 缓存键稳定性 | 同参数同键；任一参数变化键变化 |
| UT-08 | 音色优先级 | 段落覆盖 > 角色 > 章节旁白 > 书籍旁白 > 系统默认 |
| UT-09 | 文件名清洗 | 非法字符替换，长度 ≤80，重名后缀 |
| UT-10 | 参数边界（rate/pitch/volume/max_chars） | 越界返回校验错误 |
| IT-01 | 登录/错误密码/限流 | 200 / 10003 / 10004 |
| IT-02 | 书籍 CRUD + 软删除 | 列表/详情/404 正确 |
| IT-03 | 封面上传与下载 | MIME、大小、鉴权正确 |
| IT-04 | 导入 UTF-8/GBK 样例 | 章节/段落数量与内容正确 |
| IT-05 | 章节编辑保护 | 有音频时 409 |
| IT-06 | 段落编辑失效 | 段落 pending、章节音频清空 |
| IT-07 | 角色识别幂等 | 重跑不重名 |
| IT-08 | 批量段落更新事务 | 一条失败全部回滚 |
| IT-09 | TTS 缓存命中 | Provider 调用 0 次 |
| IT-10 | TTS 部分失败 | 任务 partial，失败段可重试 |
| IT-11 | 任务取消 | 停止新增调用，状态 canceled |
| IT-12 | Worker 崩溃回收 | 90s 内重新入队并完成 |
| IT-13 | 拼接（fixture） | 时长/响度/ID3 正确 |
| IT-14 | 拼接失败保留旧音频 | 旧 `audio_file_id` 不变 |
| IT-15 | 导出 ZIP | 文件数/manifest/命名正确 |
| IT-16 | SSE 终态关闭 | 收到 done 后流结束 |
| IT-17 | 重复任务 | 409/40004 且返回 existing_job_id |
| IT-18 | 未认证访问 | 401/10001 |

### 3.4 测试环境

1. CI 使用 MySQL 8 服务容器，创建独立测试库 `novelcast_test`。
2. 每个测试用例使用唯一书籍/用户数据，结束后清理；或每用例事务回滚（API 测试不适用）。
3. FFmpeg 在 CI 镜像中预装；无 FFmpeg 时拼接测试标记为跳过并输出警告。
4. 真实 TTS 接口需要 API Key，默认不进入 CI；由手工验收覆盖。

### 3.5 CI 流水线

```text
1. cargo fmt --check
2. cargo clippy --all-targets -- -D warnings
3. cargo test（MySQL service + 环境变量）
4. cargo build --release
5. docker build（backend/Dockerfile）
6. 可选：cargo audit
```

## 4. 手工验收脚本（M6 使用）

1. `docker compose up -d`，确认 `/readyz` 全绿。
2. 登录获取 Token。
3. 上传 `fixtures/novels/sample_utf8.txt`，等待导入任务成功。
4. 查看章节、段落、角色列表。
5. 登记一个音色（OpenAI/MiniMax/阶跃），试听。
6. 给角色绑定音色，设置旁白音色。
7. 对第 1 章发起批量合成，观察 SSE 进度到 100%。
8. 拼接第 1 章，下载并播放，检查时长/停顿/响度。
9. 导出整书 ZIP，解压检查目录结构与 `manifest.json`。
10. 重复步骤 7，确认缓存命中、无重复 Provider 调用。
11. 修改一段文本，确认该段回到 pending、章节音频失效。
12. 取消一个运行中的任务，确认不再新增 Provider 请求。

## 5. 风险登记

| 编号 | 风险 | 影响 | 对策 |
| --- | --- | --- | --- |
| R-01 | TTS 供应商限流/额度/接口变更 | 合成失败、任务 partial | Provider 统一接口；限速；错误透传；可切换供应商 |
| R-02 | 编码识别失败（罕见编码/混合编码） | 导入乱码或失败 | 严格 UTF-8 → GB18030 → 替换率检查；提示用户转码 |
| R-03 | 大文件导入内存占用 | OOM | 50MB 上限；按章分批入库；不整书构建大对象 |
| R-04 | FFmpeg 版本差异导致参数/编码不一致 | 拼接失败或音质差异 | Docker 镜像固定版本；启动检查；参数集中封装 |
| R-05 | RustFS S3 兼容性差异 | 上传/下载失败 | Path-Style；启动自检；集成测试；配置可切换 |
| R-06 | 任务重复执行/重复计费 | 资源浪费 | 段落 CAS；重复任务 409；缓存键去重 |
| R-07 | Worker 崩溃导致任务卡住 | 进度停滞 | 心跳 + 90s 回收；幂等重跑 |
| R-08 | 对象存储孤儿文件 | 存储膨胀 | 记录日志；Phase 2 维护任务清理 7 天前无引用文件 |
| R-09 | 章节音频/ID3 与文本不同步 | 用户听到旧内容 | 编辑失效矩阵强制清空章节音频 |
| R-10 | 单管理员密码泄露 | 全量数据风险 | 强密码策略；登录限流；部署内网；后续支持改密 |
| R-11 | MySQL 队列在任务量大时轮询开销 | CPU/IO 升高 | 索引；1s 轮询；多 Worker 时 SKIP LOCKED；后续可换 Redis |
| R-12 | 角色识别准确率不足 | 人工返工 | 置信度阈值 + 待确认列表 + 人工覆盖；后续可接 LLM |

## 6. 阶段进入下一阶段的门槛

| 门槛 | 要求 |
| --- | --- |
| G0 → M1 | M0 验收全部通过，Docker 一键启动 |
| M1 → M2 | 认证与书籍链路可用，鉴权测试通过 |
| M2 → M3 | 导入流程端到端可用，SSE 可用 |
| M3 → M4 | 角色/音色数据链路可用 |
| M4 → M5 | Mock 合成全流程可用，队列可靠性测试通过 |
| M5 → M6 | 真实拼接/导出产物可播放、可解压 |
| 交付 | M6 验收 + 手工验收脚本全部通过 + 文档一致 |

## 7. 评审关注点

1. 阶段划分是否合理，是否有需要提前/合并的阶段。
2. 测试范围是否覆盖你关心的风险点。
3. 是否接受「M1–M5 用 Mock Provider 测试，真实 TTS 仅手工验收」。
4. 是否需要在 M6 增加性能压测（100 万字导入、500 段合成）。
