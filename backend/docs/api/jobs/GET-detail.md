# 任务详情

查询单个异步任务的详细信息，可选携带任务子项（逐段落 / 逐章节明细）与任务结果。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/jobs/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 任务 ID |

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| include_items | boolean | 否 | false | 是否返回任务子项明细（最多 500 条）；`false` 时 `items` 为空数组 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（JobDetail）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job | object | 任务摘要，结构见下表 |
| items | array | 任务子项列表（`include_items=true` 时返回，最多 500 条），元素结构见下表 |
| result | object / null | 任务完成后的结果数据，结构随任务类型不同；未完成为 `null` |

`job` 对象字段（JobSummary）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 任务 ID |
| job_type | string | 任务类型：`import_parse` / `chapter_tts` / `chapter_merge` / `character_detect` / `book_export` |
| status | string | 任务状态：`queued` / `running` / `succeeded` / `partial` / `failed` / `canceled` |
| book_id | integer / null | 关联书籍 ID |
| chapter_id | integer / null | 关联章节 ID（章节级任务） |
| progress | integer | 进度百分比（0~100） |
| done_items | integer | 已完成子项数 |
| failed_items | integer | 失败子项数 |
| total_items | integer | 子项总数 |
| current_step | string / null | 当前步骤描述，如「合成 5/12」 |
| error_message | string / null | 失败原因 |
| created_at | string | 创建时间（ISO 8601 UTC） |
| started_at | string / null | 开始执行时间 |
| finished_at | string / null | 结束时间 |

`items[]` 元素字段（JobItemDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| item_type | string | 子项类型，如 `segment` |
| item_id | integer | 子项资源 ID（如段落 ID） |
| status | string | 子项状态，如 `pending`（待处理）、`failed`（失败） |
| attempts | integer | 已尝试次数 |
| error_message | string / null | 子项失败原因 |

`result` 按任务类型：

| job_type | result 字段 |
| --- | --- |
| import_parse | `book_id`、`chapters`（章节数）、`segments`（段落数）、`encoding`（源文件编码） |
| chapter_tts | `done`（成功数）、`failed`（失败数）、`cached`（缓存命中数） |
| chapter_merge | `chapter_id`、`file_id`、`duration_ms`、`skipped_segments`（实际拼接段落数） |
| character_detect | `book_id`、`characters_created`、`segments_assigned`、`segments_uncertain` |
| book_export | `export_id`、`file_id`、`included_chapters`、`size_bytes` |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "job": {
      "id": 202,
      "job_type": "chapter_tts",
      "status": "succeeded",
      "book_id": 1,
      "chapter_id": 10,
      "progress": 100,
      "done_items": 12,
      "failed_items": 0,
      "total_items": 12,
      "current_step": "合成 12/12",
      "error_message": null,
      "created_at": "2026-11-13T09:00:00Z",
      "started_at": "2026-11-13T09:00:02Z",
      "finished_at": "2026-11-13T09:02:30Z"
    },
    "items": [
      {
        "item_type": "segment",
        "item_id": 100,
        "status": "done",
        "attempts": 1,
        "error_message": null
      }
    ],
    "result": {
      "done": 12,
      "failed": 0,
      "cached": 5
    }
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 任务不存在 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/jobs/202?include_items=true' \
  -H 'Authorization: Bearer <access_token>'
```