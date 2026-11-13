# 创建整书导出

将书籍已拼接（`merged`）的章节音频打包为 ZIP（含 `manifest.json` 清单、可选封面），投递异步导出任务（`book_export`）。接口立即返回 `202` 与导出记录 ID、任务 ID。

**前置条件**：书籍至少有一个已拼接章节；指定 `chapter_ids` 时，所有 ID 必须存在且已拼接。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books/{id}/exports` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

### Body 参数（JSON，全部可选）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| include_cover | boolean | 否 | true | 是否在 ZIP 中包含封面图片（书籍未上传封面时自动忽略） |
| chapter_ids | array[integer] | 否 | 全部已拼接章节 | 指定导出的章节 ID 列表；每个 ID 必须存在且已拼接，否则返回 `20003` |

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| export_id | integer | 导出记录 ID，用于查询导出状态与下载 |
| job_id | integer | 导出任务 ID（`job_type` 为 `book_export`） |

任务完成后，任务的 `result` 字段包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| export_id | integer | 导出记录 ID |
| file_id | integer | 生成的 ZIP 文件 ID |
| included_chapters | integer | 实际导出的章节数 |
| size_bytes | integer | ZIP 文件大小（字节） |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "export_id": 7,
    "job_id": 401
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在 |
| 409 | 20003 | 没有已拼接章节；指定的部分章节未拼接或不存在；没有可导出章节 |
| 409 | 40004 | 该书籍已有进行中的导出任务，`details.existing_job_id` 为进行中任务 ID |

## curl 示例

导出全部已拼接章节（含封面）：

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/exports' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"include_cover": true}'
```

仅导出指定章节、不含封面：

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/exports' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "include_cover": false,
    "chapter_ids": [10, 11, 12]
  }'
```

> ZIP 内包含 `manifest.json`（书籍信息、导出时间、章节清单与时长）、各章 MP3（`{三位序号}-{标题}.mp3`）及可选封面。导出完成后通过 `GET /api/v1/exports/{export_id}/download` 下载。