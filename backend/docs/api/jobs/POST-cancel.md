# 取消任务

取消排队中或执行中的任务。排队中（`queued`）的任务立即取消；执行中（`running`）的任务标记取消请求，由 Worker 在下一个检查点停止。已结束的任务不能取消。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/jobs/{id}/cancel` |
| 认证 | Bearer Token |
| Content-Type | 无请求体（可发送空 JSON `{}`） |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 任务 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 任务 ID |
| status | string | 取消后的状态：排队中任务直接变为 `canceled`；执行中任务仍为 `running`（等待 Worker 停止） |
| cancel_requested | boolean | 是否已标记取消请求：排队中取消为 `false`（已直接取消）；执行中取消为 `true` |

## 响应示例

取消排队中的任务：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 202,
    "status": "canceled",
    "cancel_requested": false
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

取消执行中的任务：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 202,
    "status": "running",
    "cancel_requested": true
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 任务不存在 |
| 409 | 20003 | 任务已结束（succeeded / partial / failed / canceled），不能取消 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/jobs/202/cancel' \
  -H 'Authorization: Bearer <access_token>'
```

> 执行中任务被取消后，最终状态为 `canceled`，可通过任务详情或 SSE 事件流确认。