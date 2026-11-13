# NovelCast（声书工坊）

中文小说转多角色有声书的后端制作平台（前后端分离，本期仅后端）。

技术栈：Rust · Axum · MySQL 8 · RustFS · FFmpeg · Docker Compose · OpenAI / MiniMax / 阶跃星辰 TTS

## 项目状态

**开发中。** 已完成 M0–M5：

- M0：Cargo 双二进制、`.env` 配置加载、统一错误、请求 ID、代码内嵌建表、RustFS 客户端、Docker Compose、健康检查
- M1：管理员引导、JWT 登录、修改密码、书籍 CRUD、封面上传、文件下载、书籍统计
- M2：TXT 导入（UTF-8/GBK/BOM）、自动切章、段落切分、旁白/对白分类、章节/段落编辑、重新切段、MySQL 任务队列、Worker 分发、SSE 进度
- M3：规则角色识别、角色 CRUD、识别任务、音色手动登记（OpenAI / MiniMax / 阶跃星辰）、音色试听、音色解析
- M4：整章/单段 TTS 任务、音色解析、合成缓存、失败重试、取消、SSE 进度、段落音频入库
- M5：章节拼接、停顿与响度归一、ID3 元数据、章节音频下载/失效、整书 ZIP 导出（manifest/封面）与下载
- M5+：回收站（软删除书籍列表、恢复、彻底删除、批量操作、保留期自动清理）

后续：M6 集成测试、安全与部署加固（按 `docs/07-roadmap-testing.md` 推进）。

## 本地启动

完整启动说明、端口与请求示例见 [backend/README.md](backend/README.md)。

```bash
cd backend
cp .env.example .env
docker compose up -d mysql rustfs   # 仅启动依赖
cargo run --bin novelcast-server    # 终端 1：API
cargo run --bin novelcast-worker    # 终端 2：Worker
```

启动后访问 `http://127.0.0.1:60181/`，健康检查为 `/healthz` 与 `/readyz`。

## 设计文档

| 文档 | 内容 |
| --- | --- |
| [00 项目总览与范围](docs/00-overview.md) | 产品目标、MVP 功能清单、非功能需求 |
| [01 目录结构规划](docs/01-directory-structure.md) | 完整目录树、文件职责、命名规范 |
| [02 系统架构与技术选型](docs/02-architecture.md) | 架构、进程模型、部署拓扑、配置项、安全 |
| [03 数据库设计](docs/03-database-design.md) | 11 张表、状态枚举、迁移计划 |
| [04 API 接口设计](docs/04-api-design.md) | 51 个接口、错误码、SSE 协议 |
| [05 核心业务流程](docs/05-core-workflows.md) | 导入、切段、角色识别、TTS、拼接、导出分步流程 |
| [06 后端模块详细设计](docs/06-backend-modules.md) | 功能模块、关键类型、方法步骤 |
| [07 开发计划与测试验收](docs/07-roadmap-testing.md) | M0–M6 阶段、测试方案、风险登记 |
| [08 评审确认单](docs/08-review-checklist.md) | 待确认决策与审核清单 |
| [09 功能优先目录结构评审](docs/09-functional-directory-tree-review.md) | 目录重组评审稿 |
| [10 回收站设计](docs/10-recycle-bin-design.md) | 软删除、恢复、彻底删除、保留期自动清理与接口路由 |

当前代码按 `docs/01-directory-structure.md` 的功能模块结构开发；后续阶段按 `docs/07-roadmap-testing.md` 推进。
