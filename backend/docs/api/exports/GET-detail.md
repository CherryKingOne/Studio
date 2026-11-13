# 导出记录详情

查询单条导出记录的状态与结果。导出为异步任务，创建后需轮询本接口（或任务中心接口）直到状态变为 `succeeded` / `failed`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/exports/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 导出记录 ID（创建导出接口返回的 `export_id`） |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（ExportDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 导出记录 ID |
| book_id | integer | 所属书籍 ID |
| export_type | string | 导出类型，固定为 `book_zip` |
| status | string | 导出状态：`queued`（排队）/ `running`（执行中）/ `succeeded`（成功）/ `failed`（失败） |
| total_chapters | integer | 书籍已拼接章节总数（创建时快照） |
| included_chapters | integer | 实际纳入导出的章节数 |
| file_id | integer / null | 生成的 ZIP 文件 ID，未完成为 `null` |
| download_url | string / null | 下载地址（`/api/v1/exports/{id}/download`），未完成为 `null` |
| error_message | string / null | 失败原因，无错误为 `null` |
| created_at | string | 创建时间（ISO 8601 UTC） |
| finished_at | string / null | 完成时间（ISO 8601 UTC），未完成为 `null` |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 7,
    "book_id": 1,
    "export_type": "book_zip",
    "status": "succeeded",
    "total_chapters": 20,
    "included_chapters": 20,
    "file_id": 88,
    "download_url": "/api/v1/exports/7/download",
    "error_message": null,
    "created_at": "2026-11-13T09:30:00Z",
    "finished_at": "2026-11-13T09:31:20Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 导出记录不存在 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/exports/7' \
  -H 'Authorization: Bearer <access_token>'
```