# 单段落 TTS 合成

为单个段落投递语音合成任务（异步，任务类型仍为 `chapter_tts`，仅包含该段落）。常用于试听调整后重新合成某一段。接口立即返回 `202`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/segments/{id}/tts` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 段落 ID |

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| force | boolean | 否 | false | 是否强制重新合成：`true` 时忽略 TTS 缓存实际调用引擎；`false` 时命中缓存直接复用 |

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 合成任务 ID（`job_type` 为 `chapter_tts`） |
| total_candidates | integer | 纳入任务的段落数（正常为 1；无法解析音色时为 0 并返回 409） |
| cached_hits | integer | 预计命中缓存的段落数（0 或 1；`force=true` 时为 0） |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "job_id": 203,
    "total_candidates": 1,
    "cached_hits": 0
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 段落或所属章节不存在 |
| 409 | 20003 | 段落没有可用音色（请先登记音色） |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/segments/100/tts' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"force": true}'
```

合成完成后试听段落音频：

```bash
curl -X GET 'http://localhost:60181/api/v1/segments/100/audio' \
  -H 'Authorization: Bearer <access_token>' \
  -o segment-100.mp3
```