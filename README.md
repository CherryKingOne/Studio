# NovelCast（声书工坊）

中文小说转多角色有声书的后端制作平台（前后端分离，本期仅后端）。

技术栈：Rust · Axum · MySQL 8 · RustFS · FFmpeg · Docker Compose

## 项目状态

**开发中。** 已完成 M0 工程基建与 M1 基础接口：

- M0：Cargo 双二进制、配置加载、统一错误、请求 ID、MySQL 迁移、RustFS 客户端、Docker Compose、健康检查
- M1：管理员引导、JWT 登录、修改密码、书籍 CRUD、封面上传、文件下载、书籍统计

尚未实现：导入解析、角色识别、TTS、拼接、导出、Worker 任务分发（按 `docs/07-roadmap-testing.md` 逐步推进）。

## 本地启动

```bash
cd backend
cp .env.example .env
docker compose up -d mysql rustfs
cargo run --bin novelcast-server
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
