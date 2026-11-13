# 书籍制作统计

获取书籍的制作进度统计：章节 / 段落各阶段计数、失败数、音频总时长与进行中任务数。用于制作看板展示。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/books/{id}/stats` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（BookStats）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| total_chapters | integer | 章节总数 |
| segmented_chapters | integer | 已切段章节数（状态为 `segmented` / `tts_partial` / `tts_done` / `merged`） |
| tts_done_chapters | integer | 已完成全部合成的章节数（状态为 `tts_done` / `merged`） |
| merged_chapters | integer | 已拼接整章音频的章节数（状态为 `merged`） |
| total_segments | integer | 段落总数 |
| done_segments | integer | 已合成完成的段落数（状态 `done`） |
| failed_segments | integer | 合成失败的段落数（状态 `failed`） |
| total_duration_ms | integer | 已拼接章节音频总时长（毫秒） |
| active_job_count | integer | 该书籍进行中的任务数（状态 `queued` / `running`） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "total_chapters": 20,
    "segmented_chapters": 20,
    "tts_done_chapters": 15,
    "merged_chapters": 10,
    "total_segments": 480,
    "done_segments": 400,
    "failed_segments": 3,
    "total_duration_ms": 3600000,
    "active_job_count": 1
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已软删除 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/books/1/stats' \
  -H 'Authorization: Bearer <access_token>'
```