# 章节批量 TTS 合成

为章节下的段落批量投递语音合成任务（`chapter_tts`，异步）。按 `mode` 选择待合成段落，支持缓存命中跳过与强制重合成。接口立即返回 `202`，进度通过任务中心接口查询。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/chapters/{id}/tts` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 章节 ID |

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| mode | string | 否 | missing | 合成范围：`missing`（状态为 `pending` / `failed` 的段落）、`failed`（仅 `failed` 段落）、`all`（全部段落） |
| force | boolean | 否 | false | 是否强制重新合成：`true` 时忽略 TTS 缓存，全部实际调用引擎；`false` 时命中缓存的段落直接复用 |

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 合成任务 ID（`job_type` 为 `chapter_tts`） |
| total_candidates | integer | 实际纳入任务的段落数（已排除无法解析音色的段落） |
| cached_hits | integer | 预计命中缓存的段落数（`force=true` 时为 0） |

任务完成后，任务的 `result` 字段包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| done | integer | 成功合成段落数（含缓存命中） |
| failed | integer | 失败段落数 |
| cached | integer | 实际命中缓存段落数 |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "job_id": 202,
    "total_candidates": 12,
    "cached_hits": 5
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | mode 不是 `missing` / `failed` / `all` |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节不存在 |
| 409 | 20003 | 没有需要合成的段落；或没有可用音色的段落（请先登记音色） |
| 409 | 40004 | 该章节已有进行中的合成任务，`details.existing_job_id` 为进行中任务 ID |

## curl 示例

默认合成缺失与失败段落：

```bash
curl -X POST 'http://localhost:60181/api/v1/chapters/10/tts' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"mode": "missing"}'
```

强制全部重新合成（忽略缓存）：

```bash
curl -X POST 'http://localhost:60181/api/v1/chapters/10/tts' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"mode": "all", "force": true}'
```

> 合成完成后章节状态更新为 `tts_done`（全部成功）或 `tts_partial`（存在失败），并使整章拼接音频失效。