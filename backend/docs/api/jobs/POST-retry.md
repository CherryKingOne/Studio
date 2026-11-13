# 重试任务

基于已结束的任务创建一个新任务（复制任务类型与参数）。仅 `failed`（失败）、`partial`（部分成功）、`canceled`（已取消）的任务可重试。可选仅重试失败子项。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/jobs/{id}/retry` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 原任务 ID |

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| retry_failed_only | boolean | 否 | false | 是否仅重试原任务中失败的子项（如失败段落）；`false` 时按原任务参数整体重跑 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 新创建的任务 ID |
| source_job_id | integer | 原任务 ID |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "job_id": 205,
    "source_job_id": 202
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 原任务不存在 |
| 409 | 20003 | 原任务状态不是 failed / partial / canceled（仅失败、部分成功或已取消的任务可重试） |

## curl 示例

整体重试：

```bash
curl -X POST 'http://localhost:60181/api/v1/jobs/202/retry' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"retry_failed_only": false}'
```

仅重试失败子项：

```bash
curl -X POST 'http://localhost:60181/api/v1/jobs/202/retry' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"retry_failed_only": true}'
```