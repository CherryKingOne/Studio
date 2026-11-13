# 任务进度事件流（SSE）

以 Server-Sent Events（SSE）实时推送任务进度。服务端每 800ms 轮询一次任务，状态有变化时推送事件；任务到达终态（`succeeded` / `partial` / `failed` / `canceled`）后推送 `done` 事件并关闭流。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/jobs/{id}/events` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |
| 响应类型 | `text/event-stream`（SSE 长连接，含 keep-alive 心跳） |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 任务 ID |

## 响应参数

SSE 事件流，每个事件包含 `event`（事件名）与 `data`（JSON 字符串）：

| 事件名 | 触发条件 | 说明 |
| --- | --- | --- |
| progress | 任务快照发生变化且未到终态 | 进度更新 |
| done | 任务到达终态 | 最后一条事件，随后关闭流 |
| error | 流期间读取任务失败 | `data` 为错误描述文本（非 JSON），随后关闭流 |

`progress` / `done` 事件的 `data` JSON 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 任务 ID |
| status | string | 任务状态：`queued` / `running` / `succeeded` / `partial` / `failed` / `canceled` |
| progress | integer | 进度百分比（0~100） |
| done_items | integer | 已完成子项数 |
| failed_items | integer | 失败子项数 |
| total_items | integer | 子项总数 |
| current_step | string / null | 当前步骤描述 |
| error_message | string / null | 失败原因 |
| updated_at | string | 任务更新时间（ISO 8601 UTC） |

## 响应示例

```
event: progress
data: {"job_id":202,"status":"running","progress":45,"done_items":5,"failed_items":0,"total_items":12,"current_step":"合成 5/12","error_message":null,"updated_at":"2026-11-13T09:01:00Z"}

event: progress
data: {"job_id":202,"status":"running","progress":90,"done_items":11,"failed_items":0,"total_items":12,"current_step":"合成 11/12","error_message":null,"updated_at":"2026-11-13T09:02:00Z"}

event: done
data: {"job_id":202,"status":"succeeded","progress":100,"done_items":12,"failed_items":0,"total_items":12,"current_step":"合成 12/12","error_message":null,"updated_at":"2026-11-13T09:02:30Z"}
```

## 错误码

连接建立前的错误以普通 JSON 返回：

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 任务不存在 |

## curl 示例

```bash
curl -N -X GET 'http://localhost:60181/api/v1/jobs/202/events' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Accept: text/event-stream'
```

> `-N` 禁用缓冲，实时输出事件。浏览器端可使用 `EventSource`（需自行解决 Authorization 头传递，例如通过代理注入或使用支持自定义头的 SSE 客户端库）。