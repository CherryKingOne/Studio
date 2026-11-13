# 10 回收站设计

> 状态：已实现（后端）。本文是回收站的数据模型、接口、路由与流程的完整设计说明。
>
> 相关文档：[03 数据库设计](03-database-design.md)、[04 API 接口设计](04-api-design.md)、[06 后端模块设计](06-backend-modules.md)。

## 1. 背景

书籍删除在 M1 就已经采用软删除：`DELETE /api/v1/books/{id}` 只写入 `books.deleted_at`，不删除章节、段落、角色、任务和对象存储文件。但当时只完成了“删除”，没有完成“删除之后怎么办”：

- 没有接口能查看已删除的书籍；
- 没有接口能恢复误删的书籍；
- 没有接口能彻底删除，数据会永久残留在库里；
- 回收站中的书籍虽然详情返回 404，但它的章节、段落、角色仍可能通过 ID 直接读写；
- 删除操作没有记录操作人，无法审计；
- 文档把 `status=archived` 和软删除混为一谈，实现与文档不一致。

本设计补齐完整的回收站闭环：**删除 → 回收站查看 → 恢复 / 彻底删除 → 保留期自动清理**。

## 2. 目标与非目标

### 2.1 目标

| 编号 | 目标 |
| --- | --- |
| R-01 | 提供回收站列表：分页、关键词、删除时间、删除人、剩余保留天数 |
| R-02 | 提供恢复接口：支持保留原书名、指定新书名、自动重命名三种冲突策略 |
| R-03 | 提供彻底删除接口：删除书籍及级联数据、未被共享的文件记录与对象存储文件 |
| R-04 | 提供批量恢复、批量彻底删除、清空回收站 |
| R-05 | 删除时记录 `deleted_by`，删除时间使用 UTC |
| R-06 | 回收站中的书籍及其章节/段落/角色对业务接口不可见（统一 404） |
| R-07 | 可配置保留期，Worker 定时清理过期项 |
| R-08 | 所有写操作具备幂等/并发保护，避免恢复与彻底删除互相踩踏 |

### 2.2 非目标

- 本章不把章节、段落、角色做成独立的回收站条目。它们从属于书籍：书进回收站，内容一起冻结；书恢复，内容一起恢复；书彻底删除，内容一起物理删除。
- 不做按用户隔离的回收站。当前 MV3 只有管理员与普通用户，尚未实现资源级权限；回收站沿用全局可见范围。
- 不实现异步彻底删除任务。单本书的物理删除在当前数据规模下同步完成；超大数据量再迁移到任务中心。
- 不恢复对象存储中的历史版本（RustFS 版本控制未启用）。

## 3. 核心概念

| 概念 | 含义 | 数据表现 |
| --- | --- | --- |
| 软删除 / 进回收站 | 书籍从正常业务中移除，但数据保留 | `books.deleted_at IS NOT NULL` |
| 回收站 | 所有 `deleted_at IS NOT NULL` 的书籍集合 | 按 `deleted_at DESC` 排序 |
| 恢复 | 书籍回到正常业务列表 | `deleted_at = NULL, deleted_by = NULL` |
| 彻底删除 | 物理删除书籍及级联数据 | `DELETE FROM books` + 文件清理 |
| 保留期 | 回收站中的最长保留时间 | `RECYCLE_BIN_RETENTION_DAYS`，默认 30 天 |
| 到期时间 | `deleted_at + 保留期` | 由接口计算，不额外落库 |

### 3.1 状态机

```text
                  DELETE /books/{id}
   active ───────────────────────────────► trashed
     ▲                                       │
     │  POST /recycle-bin/book/{id}/restore  │
     └───────────────────────────────────────┘
                                             │
                       DELETE /recycle-bin/book/{id}
                                             │  或保留期到期（Worker）
                                             ▼
                                          purged
```

- `active → trashed`：软删除，记录 `deleted_at` 与 `deleted_by`。
- `trashed → active`：恢复，恢复后 `deleted_at`、`deleted_by` 置空。
- `trashed → purged`：彻底删除，不可逆。
- 只有 `trashed` 状态允许恢复和彻底删除；对 `active` 书籍执行彻底删除返回 `409/20003`。
- 重复恢复：第二次恢复返回 `404/20001`（已不在回收站）。
- 重复彻底删除：第二次返回 `404/20001`。

### 3.2 与归档的区别

`books.status` 是业务状态（`draft`、`importing`、`ready`、`failed` 等）。`archived` 仍然只是一个业务状态，**不是删除标记**。

- 软删除只写 `deleted_at`，不修改 `status`；
- 恢复后书籍回到删除前的 `status`；
- `GET /books` 的 `include_archived` 只影响 `status = 'archived'` 的书籍，与回收站无关。

## 4. 数据模型

### 4.1 `books` 表变更

| 字段 | 类型 | 允许空 | 说明 |
| --- | --- | --- | --- |
| `deleted_at` | DATETIME(3) | 是 | 软删除时间，UTC；已有字段 |
| `deleted_by` | BIGINT UNSIGNED | 是 | 删除人 `users.id`；新增 |

索引与外键：

- `KEY idx_books_deleted_at (deleted_at)`：已有，服务回收站列表与过期扫描；
- `KEY idx_books_deleted_by (deleted_by)`：新增；
- `CONSTRAINT fk_books_deleted_by FOREIGN KEY (deleted_by) REFERENCES users (id) ON DELETE SET NULL`：删除人注销后置空，不影响回收站清理。

迁移版本：`0003_book_deleted_by`，SQL 内嵌在 `backend/src/shared/schema.rs`。

> 不新增 `status='deleted'`。删除标记与业务状态分离，避免恢复时丢失原状态。

### 4.2 级联关系

书籍进入回收站时不会触发数据库级联；只有彻底删除时才会触发：

| 关联表 | 外键 | 彻底删除时 |
| --- | --- | --- |
| `chapters` | `book_id ON DELETE CASCADE` | 物理删除 |
| `characters` | `book_id ON DELETE CASCADE` | 物理删除 |
| `segments` | `chapter_id ON DELETE CASCADE` | 随章节物理删除 |
| `exports` | `book_id ON DELETE CASCADE` | 物理删除 |
| `jobs` | `book_id ON DELETE SET NULL` | 先显式删除该书籍的任务，再删书籍 |
| `job_items` | `job_id ON DELETE CASCADE` | 随任务删除 |
| `files` | 由业务表引用，`ON DELETE SET NULL` | 仅删除未被其他数据引用的文件 |
| `audio_cache` | `file_id ON DELETE CASCADE` | 随被删除的文件清理缓存行 |

### 4.3 文件清理策略

彻底删除时按以下顺序处理：

1. 收集书籍直接关联的文件 ID：封面、导入源文件、章节音频、段落音频、导出 ZIP；
2. 删除该书籍的 `jobs`；
3. 删除 `books` 行，级联章节、角色、段落、导出；
4. 重新查询这些文件是否仍被其他数据引用：其他书籍封面/源文件、章节音频、段落音频、音色样例、导出文件；
5. 只删除**未被引用**的 `files` 行；
6. 事务提交后，尽力删除对象存储中的对应对象。

被其他书籍共享的音频缓存文件会保留，避免破坏其他书籍的音频。对象存储删除失败只记录日志，不回滚数据库，也不让接口失败；响应通过 `objects_failed` 给出失败数量。原因是对象存储删除不可参与 MySQL 事务，且重复删除是幂等的。

## 5. 接口与路由

所有接口统一前缀 `/api/v1`，均需要 `Authorization: Bearer <access_token>`。成功响应均使用统一包络；删除类接口返回 `200 + message=deleted + 删除结果`，不使用无响应体的 `204`，便于前端确认结果和调试。

### 5.1 路由总览

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/recycle-bin` | 回收站分页列表 | 200 |
| GET | `/recycle-bin/summary` | 回收站汇总 | 200 |
| POST | `/recycle-bin/{resource_type}/{id}/restore` | 恢复单个条目 | 200 |
| DELETE | `/recycle-bin/{resource_type}/{id}` | 彻底删除单个条目 | 200 |
| POST | `/recycle-bin/restore` | 批量恢复（≤100） | 200 |
| POST | `/recycle-bin/purge` | 批量彻底删除（≤100） | 200 |
| DELETE | `/recycle-bin?confirm=true` | 清空回收站 | 200 |

`resource_type` 目前只接受 `book`。使用路径参数而不是 `/recycle-bin/books`，是为将来扩展 `chapter`、`character` 等独立回收站条目留出统一入口；在当前阶段传其他值返回 `400/20002`。

路由注册位置：`backend/src/runtime/router.rs`，实现位于 `backend/src/recycle_bin/routes.rs`。

### 5.2 GET `/recycle-bin`

查询参数：

| 参数 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- |
| `page` | integer | 否 | 1 | 页码，小于 1 按 1 |
| `page_size` | integer | 否 | 20 | 1–100，超过按 100 |
| `keyword` | string | 否 | — | 模糊匹配书名或作者 |

说明：

- 只返回 `deleted_at IS NOT NULL` 的书籍；
- 排序固定为 `deleted_at DESC, id DESC`；
- `remaining_days` 向上取整，已到期为 `0`；
- 保留期为 `0` 时，`expires_at` 和 `remaining_days` 为 `null`，表示永久保留。

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "resource_type": "book",
        "id": 12,
        "title": "山海拾遗",
        "author": "某某",
        "cover_url": "/api/v1/files/88",
        "status": "ready",
        "total_chapters": 36,
        "total_segments": 842,
        "total_duration_ms": 5400000,
        "deleted_at": "2026-11-13T08:00:00Z",
        "deleted_by": {
          "id": 1,
          "username": "admin",
          "display_name": "管理员"
        },
        "expires_at": "2026-12-13T08:00:00Z",
        "remaining_days": 30
      }
    ],
    "page": 1,
    "page_size": 20,
    "total": 1
  },
  "request_id": "..."
}
```

`deleted_by` 在历史数据没有记录删除人、或删除人已被删除时为 `null`。

### 5.3 GET `/recycle-bin/summary`

用于回收站入口角标与保留期提示。

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "total": 3,
    "resource_types": [
      { "resource_type": "book", "count": 3 }
    ],
    "retention_days": 30,
    "oldest_deleted_at": "2026-11-01T08:00:00Z",
    "next_expire_at": "2026-12-01T08:00:00Z"
  },
  "request_id": "..."
}
```

- `total`：回收站条目总数；
- `resource_types`：按资源类型计数，当前只有 `book`；
- `retention_days`：当前配置的保留天数，`0` 表示永久保留；
- `oldest_deleted_at`：最早删除时间，空回收站为 `null`；
- `next_expire_at`：最早到期时间，保留期为 `0` 或空回收站为 `null`。

### 5.4 POST `/recycle-bin/{resource_type}/{id}/restore`

请求体可选；不传 `Content-Type` / 请求体时按默认策略恢复。

| 字段 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- |
| `title` | string | 否 | 原书名 | 恢复时使用的新书名，1–255 字符 |
| `on_conflict` | string | 否 | `fail` | `fail` 或 `rename` |

冲突规则：

| 场景 | 行为 |
| --- | --- |
| 原书名未被占用 | 使用原书名恢复 |
| 显式指定 `title` 且被占用 | `409/20004`，不自动改名 |
| 未指定 `title` 且原书名被占用，`on_conflict=fail` | `409/20004`，提示指定新书名或使用 `rename` |
| 未指定 `title` 且原书名被占用，`on_conflict=rename` | 自动生成 `书名（恢复）`、`书名（恢复 2）`…… |

自动改名会截断原书名，保证总长度不超过 255 字符。

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "resource_type": "book",
    "id": 12,
    "book_id": 12,
    "title": "山海拾遗（恢复）",
    "status": "ready",
    "restored_at": "2026-11-13T09:00:00Z"
  },
  "request_id": "..."
}
```

错误：

- `404/20001`：书籍不存在，或已不在回收站；
- `409/20004`：书名冲突且策略不允许改名；
- `400/20002`：`title` 为空/超长，或 `on_conflict` 非法。

### 5.5 DELETE `/recycle-bin/{resource_type}/{id}`

彻底删除单个回收站条目，不可恢复。

- 成功：HTTP `200`，`message=deleted`，返回删除结果；
- 只允许删除 `deleted_at IS NOT NULL` 的书籍；
- 存在 `queued/running` 任务时拒绝删除；
- 删除数据库数据、该书籍任务，以及未被共享的文件记录和对象存储对象；
- 对象存储删除失败不回滚，不改变成功状态。

响应 `200`：

```json
{
  "code": 0,
  "message": "deleted",
  "data": {
    "resource_type": "book",
    "id": 12,
    "purged": true,
    "files_deleted": 4,
    "objects_failed": 0
  },
  "request_id": "..."
}
```

- `files_deleted`：本次删除的 `files` 记录数；
- `objects_failed`：对象存储删除失败数，仅告警，不影响 `purged=true`。

错误：

| HTTP | code | 场景 |
| --- | --- | --- |
| 404 | 20001 | 书籍不存在，或已彻底删除 |
| 409 | 20003 | 书籍在正常列表中（不在回收站），或存在进行中任务 |
| 400 | 20002 | `resource_type` 不是 `book` |

> 对正常书籍执行彻底删除会返回 `409/20003`，而不是隐式先软删除，避免误删。

### 5.6 POST `/recycle-bin/restore`

批量恢复。单个失败不阻断其他条目，整体返回 `200`，逐项给出结果。

请求：

```json
{
  "items": [
    { "resource_type": "book", "id": 12 },
    { "resource_type": "book", "id": 13 }
  ],
  "on_conflict": "rename"
}
```

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `items` | array | 是 | 1–100 条，不允许重复 ID |
| `items[].resource_type` | string | 是 | 当前只接受 `book` |
| `items[].id` | integer | 是 | 书籍 ID |
| `on_conflict` | string | 否 | 同单条恢复，默认 `fail` |

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "restored": 1,
    "failed": 1,
    "items": [
      {
        "resource_type": "book",
        "id": 12,
        "success": true,
        "title": "山海拾遗",
        "code": null,
        "message": null
      },
      {
        "resource_type": "book",
        "id": 13,
        "success": false,
        "title": null,
        "code": 20004,
        "message": "同名书籍「山海拾遗」已存在，请指定新书名或设置 on_conflict=rename"
      }
    ]
  },
  "request_id": "..."
}
```

`restored + failed = items.length`。

### 5.7 POST `/recycle-bin/purge`

批量彻底删除。语义同单条彻底删除；单条失败不阻断其他条目。

请求：

```json
{
  "items": [
    { "resource_type": "book", "id": 12 }
  ]
}
```

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "purged": 1,
    "failed": 0,
    "files_deleted": 4,
    "objects_failed": 0,
    "items": [
      {
        "resource_type": "book",
        "id": 12,
        "success": true,
        "title": null,
        "code": null,
        "message": null
      }
    ]
  },
  "request_id": "..."
}
```

- `files_deleted`：本次删除的 `files` 记录数；
- `objects_failed`：对象存储删除失败数，仅告警，不影响 `purged`。

### 5.8 DELETE `/recycle-bin?confirm=true`

清空回收站。

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `confirm` | boolean | 是 | 必须为 `true`，否则 `400/20002` |

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "purged": 2,
    "failed": 0,
    "files_deleted": 8,
    "objects_failed": 0
  },
  "request_id": "..."
}
```

清空是物理删除，不可恢复；前端必须二次确认后再传 `confirm=true`。

## 6. 模块与代码结构

```text
backend/src/recycle_bin/
├── mod.rs       # 模块导出
├── routes.rs    # 路由注册
├── handler.rs   # HTTP 参数解析与响应包装
├── dto.rs       # 请求/响应结构、保留期计算
├── repo.rs      # 回收站查询、恢复、文件引用判断、彻底删除 SQL
└── service.rs   # 业务编排：恢复、彻底删除、批量、清空、自动清理
```

分层职责：

| 层 | 职责 |
| --- | --- |
| `handler` | 鉴权提取、Path/Query/Json 解析、调用 service、统一包络 |
| `service` | 校验资源类型与批量上限、冲突策略、事务边界、对象存储清理 |
| `repo` | 所有 SQL；事务内行锁、文件引用检查 |
| `dto` | 稳定字段顺序的响应结构；`expires_at` / `remaining_days` 计算 |

依赖方向：`recycle_bin` 可以依赖 `library::books`（读取书籍模型、复用软删除字段），`library::books` 不依赖 `recycle_bin`，避免循环依赖。

## 7. 关键流程

### 7.1 删除进入回收站

沿用 M1 的 `DELETE /books/{id}`，新增记录删除人：

1. 校验书籍存在且未删除；
2. 检查 `jobs` 中是否存在 `queued/running` 任务；
3. `UPDATE books SET deleted_at = UTC_TIMESTAMP(3), deleted_by = ? WHERE id = ? AND deleted_at IS NULL`；
4. 返回 `200`，`message=deleted`，`data` 含 `id`、`title`、`deleted_at`、`deleted_by`。

不停止/删除任务与文件。正在进行的任务会阻止删除。

### 7.2 恢复

```text
BEGIN
  SELECT * FROM books WHERE id = ? AND deleted_at IS NOT NULL FOR UPDATE
  校验书名冲突；必要时自动生成“（恢复）”后缀
  UPDATE books SET title = ?, deleted_at = NULL, deleted_by = NULL, updated_at = UTC_TIMESTAMP(3)
COMMIT
返回恢复后的 status / title
```

行锁保证同一本书的恢复与彻底删除互斥。

### 7.3 彻底删除

```text
BEGIN
  SELECT ... FOR UPDATE（必须是回收站中的书）
  检查无 queued/running 任务
  收集文件 ID（封面/源文件/章节音频/段落音频/导出）
  DELETE FROM jobs WHERE book_id = ? OR chapter_id IN (...)
  DELETE FROM books WHERE id = ? AND deleted_at IS NOT NULL  -- 级联章节/段落/角色/导出
  查询这些文件是否仍被其他数据引用
  DELETE FROM files WHERE id IN (未被引用的文件)
COMMIT
逐个小对象执行对象存储删除（失败只告警）
```

### 7.4 保留期自动清理

- 配置项：`RECYCLE_BIN_RETENTION_DAYS`，默认 `30`，`0` 表示永久保留；
- 截点为 `UTC 当前时间 - 保留期`；
- Worker 每小时执行一次 `purge_expired`；
- 每次最多扫描 100 条，最多 10 轮，避免一次清理阻塞任务分发；
- 单条失败记录日志并跳过，下一轮继续；
- 没有运行 Worker 时不会自动清理，但回收站接口仍可手动清空。

## 8. 一致性与并发

| 场景 | 保障 |
| --- | --- |
| 恢复 vs 彻底删除 | 两者都在事务内 `SELECT ... FOR UPDATE` 锁书籍行 |
| 重复恢复 | `UPDATE ... WHERE deleted_at IS NOT NULL`，第二次影响 0 行 → 404 |
| 重复彻底删除 | `DELETE ... WHERE deleted_at IS NOT NULL`，第二次影响 0 行 → 404 |
| 恢复时书名并发占用 | 事务内检查；数据库当前没有书名唯一约束，极端并发下仍可能重名，后续可补唯一索引或应用层锁 |
| 彻底删除时创建新任务 | 书籍处于回收站时 `books_service::get_model` 返回 404，所有任务创建入口都无法再引用它 |
| 彻底删除与进行中任务 | 彻底删除前检查 `queued/running`，存在则 409 |
| 文件被其他书籍共享 | 删除前查询六类引用，只删除未被引用的文件 |
| 对象存储删除失败 | 数据库删除已提交；返回 `objects_failed`，日志告警，对象成为孤儿文件，可人工/离线清理 |
| 批量部分失败 | 逐项提交，失败项返回 business code，成功项不受影响 |

### 8.1 回收站期间的访问控制

书籍处于回收站时，以下入口统一按“不存在”处理（404）：

- 书籍列表 / 详情 / 更新 / 统计 / 封面；
- 章节列表 / 详情 / 编辑 / 重新切段 / TTS / 拼接 / 音频；
- 段落列表 / 编辑 / 批量编辑 / TTS / 音频；
- 角色列表 / 新建 / 编辑 / 删除 / 识别；
- 整书导出创建、导出任务执行。

实现方式：

- 章节查询统一 `JOIN books ... AND books.deleted_at IS NULL`；
- 段落查询再 `JOIN chapters`；
- 角色查询 `JOIN books`；
- 书籍相关入口继续走 `book_data::find_by_id` 的 `deleted_at IS NULL` 过滤。

导出记录详情/下载、文件下载、历史任务查询不做回收站过滤，它们属于只读产物或审计数据。

## 9. 错误码

复用全局错误码，不新增业务码：

| HTTP | code | 回收站场景 |
| --- | --- | --- |
| 400 | 20002 | 资源类型不支持、批量数量/重复项、书名非法、`on_conflict` 非法、缺少 `confirm` |
| 401 | 10001 | 未携带或无效 Token |
| 404 | 20001 | 条目不在回收站或已彻底删除 |
| 409 | 20003 | 正常书籍被执行彻底删除；存在进行中任务 |
| 409 | 20004 | 恢复书名冲突 |
| 500 | 50001 | 数据库等内部错误 |

## 10. 配置项

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `RECYCLE_BIN_RETENTION_DAYS` | `30` | 保留天数，`0`–`3650`；`0` 表示永久保留且关闭自动清理 |

启动时校验，超过 3650 直接拒绝启动。

## 11. 安全与审计

- 所有回收站接口要求登录，沿用 Bearer Token；
- 恢复、彻底删除、清空都是写操作，后续接入资源级权限时只需在 handler 增加校验；
- `deleted_by` 记录删除人，`deleted_at` 记录删除时间；
- 彻底删除时先删除 `jobs`，避免任务列表出现指向已删除书籍的“僵尸任务”；
- 清空回收站要求显式 `confirm=true`，防止误触；
- 接口不返回对象存储路径、内部 SQL 或堆栈。

## 12. 前端交互建议

1. 书籍列表/详情删除成功后，提示“已移入回收站”，并刷新列表；详情从当前页移除。
2. 回收站入口显示 `summary.total` 角标；进入后调用列表接口。
3. 列表项展示封面、书名、作者、删除时间、删除人、剩余天数；`remaining_days <= 3` 时高亮。
4. 恢复按钮调用单条恢复；若返回 `20004`，弹出“改名恢复”对话框：
   - 预填 `书名（恢复）`；
   - 用户可编辑后带 `title` 重试；
   - 或直接使用 `on_conflict=rename` 自动命名。
5. 多选恢复/彻底删除使用批量接口；根据 `items[].success` 分别提示成功与失败数量。
6. “彻底删除”和“清空回收站”必须二次确认，确认文案要说明不可恢复。
7. 保留期为 `0` 时隐藏剩余天数，展示“永久保留”。

## 13. 测试与验收

### 13.1 已覆盖的单元测试

- `expires_at` / `remaining_days` 计算（向上取整、到期为 0、保留期为 0）；
- 自动恢复后缀长度不超过 255、空书名兜底；
- `on_conflict` 解析、资源类型校验、批量重复项校验。

### 13.2 已完成的接口实测

| 用例 | 结果 |
| --- | --- |
| 空回收站 / 有数据列表 / 关键词过滤 | 通过 |
| 汇总数量、最早删除时间、最早到期时间 | 通过 |
| 软删除后详情 404、列表不返回 | 通过 |
| 回收站恢复，恢复后详情与列表可见 | 通过 |
| 同名冲突恢复 409，`on_conflict=rename` 成功并改名 | 通过 |
| 无请求体恢复 | 通过 |
| 单个彻底删除：删除书籍、封面文件记录与对象，文件 URL 变 404 | 通过 |
| 批量恢复部分失败（含不存在的 ID），逐项返回 | 通过 |
| 批量彻底删除：书籍、章节、段落、任务、导入源文件全部清理 | 通过 |
| 清空回收站（`confirm` 缺失 400，`confirm=true` 成功） | 通过 |
| 回收站期间章节/段落/角色/TTS/音频接口全部 404 | 通过 |
| 过期书籍由 Worker 自动清理 | 通过 |
| 对正常书籍执行彻底删除返回 409 | 通过 |
| 批量超过 100 条、重复 ID 返回 400 | 通过 |

### 13.3 后续自动化测试建议

- 用测试数据库补 `sqlx` 集成测试：恢复冲突、彻底删除文件共享、并发恢复/彻底删除；
- 增加路由级测试，防止未来改动 `deleted_at` 过滤条件造成回归。

## 14. 后续扩展

| 扩展 | 做法 |
| --- | --- |
| 章节/角色独立回收站 | 增加 `deleted_at` 字段与统一 `resource_type` 分发；本设计的 `/recycle-bin/{resource_type}/{id}` 已预留路径 |
| 回收站按用户隔离 | 在 `repo` 增加 `created_by/deleted_by` 过滤，并在 handler 校验权限 |
| 异步彻底删除 | 大批量清理改为 `task_center` 的 `recycle_purge` 任务，接口返回 `202` |
| 对象存储孤儿清理 | 定期扫描 `files` 与对象存储差异，补偿 `objects_failed` |
| 恢复历史 | 新增 `book_restore_logs` 审计表，记录操作人、时间、原书名/新书名 |
| 保留期按本配置 | 在 `books.settings` 增加 `retention_days` 覆盖全局配置 |

## 15. 变更清单

| 文件 | 变更 |
| --- | --- |
| `backend/src/shared/schema.rs` | 新增迁移 `0003_book_deleted_by` |
| `backend/src/library/books/book_data.rs` | `Book.deleted_by`；`soft_delete` 记录删除人 |
| `backend/src/library/books/book_actions.rs` | 删除接口传递 `user.id` |
| `backend/src/library/books/api.rs` | 删除 handler 使用 `AuthUser` |
| `backend/src/library/chapters/*` | 章节查询过滤回收站书籍 |
| `backend/src/library/segments/segment_data.rs` | 段落单条查询过滤回收站书籍 |
| `backend/src/casting/characters/*` | 角色查询过滤回收站书籍 |
| `backend/src/recycle_bin/*` | 回收站模块（DTO / repo / service / handler / routes） |
| `backend/src/runtime/router.rs` | 注册回收站路由 |
| `backend/src/runtime/config.rs` | `RECYCLE_BIN_RETENTION_DAYS` |
| `backend/src/runtime/error.rs` | 暴露业务错误码给批量结果 |
| `backend/src/worker/dispatcher.rs` | 每小时自动清理过期回收站条目 |
| `backend/docs/api/recycle-bin/*` | 7 个接口文档 |
| `docs/03-database-design.md`、`docs/04-api-design.md` | 同步数据模型与接口 |
