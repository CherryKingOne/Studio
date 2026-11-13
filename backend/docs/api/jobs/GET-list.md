# 任务列表

分页查询异步任务，按创建时间倒序排列，支持按书籍、任务类型、状态筛选。用于任务中心看板。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/jobs` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| page | integer | 否 | 1 | 页码，从 1 开始，小于 1 按 1 处理 |
| page_size | integer | 否 | 20 | 每页条数，限制 1~100 |
| book_id | integer | 否 | - | 按关联书籍 ID 精确筛选 |
| job_type | string | 否 | - | 按任务类型精确筛选：`import_parse` / `chapter_tts` / `chapter_merge` / `character_detect` / `book_export` |
| status | string | 否 | - | 按状态精确筛选：`queued` / `running` / `succeeded` / `partial` / `failed` / `canceled` |

## 响应参数

HTTP `200`，统一包装结构，`data` 为分页对象：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 任务摘要列表，元素结构见下表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 满足条件的任务总数 |

`items[]` 元素字段（JobSummary）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 任务 ID |
| job_type | string | 任务类型：`import_parse`（导入解析）/ `chapter_tts`（章节合成）/ `chapter_merge`（章节拼接）/ `character_detect`（角色识别）/ `book_export`（整书导出） |
| status | string | 任务状态：`queued`（排队）/ `running`（执行中）/ `succeeded`（成功）/ `partial`（部分成功）/ `failed`（失败）/ `canceled`（已取消） |
| book_id | integer / null | 关联书籍 ID |
| chapter_id | integer / null | 关联章节 ID（章节级任务） |
| progress | integer | 进度百分比（0~100） |
| done_items | integer | 已完成子项数 |
| failed_items | integer | 失败子项数 |
| total_items | integer | 子项总数 |
| current_step | string / null | 当前步骤描述，如「切分章节 3/20」 |
| error_message | string / null | 失败原因 |
| created_at | string | 创建时间（ISO 8601 UTC） |
| started_at | string / null | 开始执行时间，未开始为 `null` |
| finished_at | string / null | 结束时间，未结束为 `null` |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 202,
        "job_type": "chapter_tts",
        "status": "running",
        "book_id": 1,
        "chapter_id": 10,
        "progress": 45,
        "done_items": 5,
        "failed_items": 0,
        "total_items": 12,
        "current_step": "合成 5/12",
        "error_message": null,
        "created_at": "2026-11-13T09:00:00Z",
        "started_at": "2026-11-13T09:00:02Z",
        "finished_at": null
      }
    ],
    "page": 1,
    "page_size": 20,
    "total": 1
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/jobs?page=1&page_size=20' \
  -H 'Authorization: Bearer <access_token>'
```

查询某本书进行中的合成任务：

```bash
curl -X GET 'http://localhost:60181/api/v1/jobs?book_id=1&job_type=chapter_tts&status=running' \
  -H 'Authorization: Bearer <access_token>'
```