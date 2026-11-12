# NovelCast（声书工坊）

中文小说转多角色有声书的后端制作平台（前后端分离，本期仅后端）。

技术栈：Rust · Axum · MySQL 8 · RustFS · FFmpeg · Docker Compose · OpenAI / MiniMax / 阶跃星辰 TTS

## 项目状态

**开发中。** 已完成 M0–M3：

- M0：Cargo 双二进制、`.env` 配置加载、统一错误、请求 ID、代码内嵌建表、RustFS 客户端、Docker Compose、健康检查
- M1：管理员引导、JWT 登录、修改密码、书籍 CRUD、封面上传、文件下载、书籍统计
- M2：TXT 导入（UTF-8/GBK/BOM）、自动切章、段落切分、旁白/对白分类、章节/段落编辑、重新切段、MySQL 任务队列、Worker 分发、SSE 进度
- M3：规则角色识别、角色 CRUD、识别任务、音色手动登记（OpenAI / MiniMax / 阶跃星辰）、音色试听、音色解析

尚未实现：TTS 批量合成、章节拼接、ZIP 导出（按 `docs/07-roadmap-testing.md` 推进）。

## 本地启动

所有配置（含 Docker Compose 变量）统一从 `backend/.env` 加载，模板为 `backend/.env.example`。使用音色试听/合成前，请填写对应的 `TTS_OPENAI_API_KEY`、`TTS_MINIMAX_API_KEY`、`TTS_STEPFUN_API_KEY`，并通过 `POST /api/v1/voices` 登记平台提供的音色 ID。

```bash
cd backend
cp .env.example .env
docker compose up -d mysql rustfs
cargo run --bin novelcast-server
```

另开终端启动 Worker（导入任务需要）：

```bash
cd backend
cargo run --bin novelcast-worker
```

健康检查：`curl http://127.0.0.1:8080/healthz` 与 `/readyz`。

## 设计文档

| 文档 | 内容 |
| --- | --- |
| [00 项目总览与范围](docs/00-overview.md) | 产品目标、MVP 功能清单、非功能需求 |
| [01 目录结构规划](docs/01-directory-structure.md) | 完整目录树、文件职责、命名规范 |
| [02 系统架构与技术选型](docs/02-architecture.md) | 架构、进程模型、部署拓扑、配置项、安全 |
| [03 数据库设计](docs/03-database-design.md) | 11 张表、状态枚举、迁移计划 |
| [04 API 接口设计](docs/04-api-design.md) | 44 个接口、错误码、SSE 协议 |
| [05 核心业务流程](docs/05-core-workflows.md) | 导入、切段、角色识别、TTS、拼接、导出分步流程 |
| [06 后端模块详细设计](docs/06-backend-modules.md) | 功能模块、关键类型、方法步骤 |
| [07 开发计划与测试验收](docs/07-roadmap-testing.md) | M0–M6 阶段、测试方案、风险登记 |
| [08 评审确认单](docs/08-review-checklist.md) | 待确认决策与审核清单 |

当前代码按 `docs/01-directory-structure.md` 的功能模块结构开发；后续阶段按 `docs/07-roadmap-testing.md` 推进。
