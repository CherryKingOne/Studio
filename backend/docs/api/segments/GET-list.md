# 段落列表

查询某章节的全部段落（不分页，按段落序号升序，单次最多返回 2000 条），支持按类型、状态、说话人筛选。每个段落附带**解析后的实际音色**（`resolved_voice`），展示合成时将使用的音色来源。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/chapters/{chapter_id}/segments` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| chapter_id | integer | 是 | 章节 ID |

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| seg_type | string | 否 | - | 按段落类型精确筛选：`narration`（旁白）/ `dialogue`（对白） |
| status | string | 否 | - | 按合成状态精确筛选：`pending` / `synthesizing` / `done` / `failed` |
| speaker_id | integer | 否 | - | 按说话人角色 ID 精确筛选 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 段落列表（最多 2000 条），元素结构见下表 |
| total | integer | 该章节段落总数（**不受筛选条件影响**） |

`items[]` 元素字段（SegmentDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 段落 ID |
| index | integer | 段落序号（章内从 1 开始） |
| text | string | 段落文本 |
| char_count | integer | 文本字符数 |
| seg_type | string | 段落类型：`narration`（旁白）/ `dialogue`（对白） |
| speaker_id | integer / null | 说话人角色 ID，未识别为 `null` |
| speaker_confidence | float / null | 说话人识别置信度（0~1）；手动指定角色时为 1.0 |
| voice_id | integer / null | 段落显式指定的音色 ID，未指定为 `null` |
| emotion | string | 情绪标签（合成参数），如 `neutral` |
| rate_percent | integer | 语速偏移（%），范围 -50~100，0 为正常 |
| pitch_hz | integer | 音调偏移（Hz），范围 -50~50，0 为正常 |
| volume_percent | integer | 音量偏移（%），范围 -50~50，0 为正常 |
| status | string | 合成状态：`pending`（待合成）/ `synthesizing`（合成中）/ `done`（已完成）/ `failed`（失败） |
| audio_duration_ms | integer / null | 段落音频时长（毫秒），未合成为 `null` |
| error_message | string / null | 最近一次合成失败原因 |
| resolved_voice | object / null | 解析后的实际音色，见下表；解析失败时为 `null` |

`resolved_voice` 对象字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| voice_id | integer / null | 解析到的音色 ID；走系统默认音色时为 `null` |
| provider | string | 合成供应商：`openai` / `minimax` / `stepfun` |
| provider_voice_id | string / null | 供应商侧音色标识（如 `alloy`）；音色缺失时为 `null` |
| display_name | string / null | 音色显示名；系统默认 / 音色缺失时为 `null` |
| source | string | 音色来源：`fallback`（段落已指定音色）、`character_default`（角色默认音色）、`book_narrator`（章节 / 书籍旁白音色）、`system_default`（系统默认音色）、`missing_voice`（指定的音色不存在或未启用） |

**音色解析优先级**：段落指定音色 → 对白角色默认音色 → 章节旁白音色（仅旁白段）→ 书籍默认旁白音色 → 系统默认音色（`TTS_DEFAULT_NARRATOR_VOICE_ID`）。

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 100,
        "index": 1,
        "text": "清晨的雾气还未散去。",
        "char_count": 11,
        "seg_type": "narration",
        "speaker_id": null,
        "speaker_confidence": null,
        "voice_id": null,
        "emotion": "neutral",
        "rate_percent": 0,
        "pitch_hz": 0,
        "volume_percent": 0,
        "status": "done",
        "audio_duration_ms": 3200,
        "error_message": null,
        "resolved_voice": {
          "voice_id": 5,
          "provider": "openai",
          "provider_voice_id": "alloy",
          "display_name": "Alloy",
          "source": "book_narrator"
        }
      }
    ],
    "total": 12
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节不存在 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/chapters/10/segments' \
  -H 'Authorization: Bearer <access_token>'
```

只看失败的对白段落：

```bash
curl -X GET 'http://localhost:60181/api/v1/chapters/10/segments?seg_type=dialogue&status=failed' \
  -H 'Authorization: Bearer <access_token>'
```