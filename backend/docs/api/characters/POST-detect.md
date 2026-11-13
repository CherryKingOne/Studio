# 角色自动识别

基于规则对全书对白段落做说话人识别：自动创建高频出现的角色（出现 ≥ 3 次），并为对白段落指派说话人。异步任务（`character_detect`），接口立即返回 `202`。

**识别规则摘要**：
- 已手动确认（置信度 ≥ 0.999）的段落不会被覆盖。
- `overwrite_existing=false` 时，已有说话人的段落跳过。
- 推断置信度 ≥ 0.6 才指派角色；否则仅记录置信度，计入「不确定」段落。
- 识别导致已合成段落说话人变化时，相关章节整章音频会被清除。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books/{book_id}/characters/detect` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| book_id | integer | 是 | 书籍 ID |

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| overwrite_existing | boolean | 否 | false | 是否覆盖已有说话人指派的段落（手动确认的高置信度段落始终不覆盖） |

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 识别任务 ID（`job_type` 为 `character_detect`） |

任务完成后，任务的 `result` 字段包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| book_id | integer | 书籍 ID |
| characters_created | integer | 新创建的角色数 |
| segments_assigned | integer | 成功指派说话人的段落数 |
| segments_uncertain | integer | 置信度不足、未指派的段落数 |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "job_id": 501
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在 |
| 409 | 40004 | 该书籍已有进行中的识别任务，`details.existing_job_id` 为进行中任务 ID |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/characters/detect' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"overwrite_existing": false}'
```

跟踪识别进度（SSE）：

```bash
curl -N -X GET 'http://localhost:60181/api/v1/jobs/501/events' \
  -H 'Authorization: Bearer <access_token>'
```

> 书籍没有段落时任务会失败（`result` 无内容，`error_message` 为「书籍没有可识别的段落」）。