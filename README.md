# NovelCast（声书工坊）

中文小说转多角色有声书的后端制作平台（前后端分离，本期仅后端）。

技术栈：Rust · Axum · MySQL 8 · RustFS · FFmpeg · Docker Compose

## 项目状态

**规划设计阶段，待评审。** 尚未创建 `backend/` 目录，未编写业务代码。

## 设计文档

| 文档 | 内容 |
| --- | --- |
| [00 项目总览与范围](docs/00-overview.md) | 产品目标、MVP 功能清单、非功能需求 |
| [01 目录结构规划](docs/01-directory-structure.md) | 完整目录树、文件职责、命名规范 |
| [02 系统架构与技术选型](docs/02-architecture.md) | 架构、进程模型、部署拓扑、配置项、安全 |
| [03 数据库设计](docs/03-database-design.md) | 11 张表、状态枚举、迁移计划 |
| [04 API 接口设计](docs/04-api-design.md) | 44 个接口、错误码、SSE 协议 |
| [05 核心业务流程](docs/05-core-workflows.md) | 导入、切段、角色识别、TTS、拼接、导出分步流程 |
| [06 后端模块详细设计](docs/06-backend-modules.md) | 分层模块、关键类型、方法步骤 |
| [07 开发计划与测试验收](docs/07-roadmap-testing.md) | M0–M6 阶段、测试方案、风险登记 |
| [08 评审确认单](docs/08-review-checklist.md) | 待确认决策与审核清单 |

审核通过后，按 `docs/01-directory-structure.md` 创建 `backend/` 并进入 M0 阶段。
