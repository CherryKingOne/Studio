# 更新段落

部分更新单个段落的文本或合成参数。仅传入需要修改的字段，未传入的字段保持原值。

**副作用（重要）**：任何更新都会将段落状态重置为 `pending`，清除段落音频与缓存键，并清除所属章节的整章拼接音频——修改后需要重新合成与拼接。段落处于 `synthesizing`（合成中）时禁止编辑。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `PATCH` |
| 请求路径 | `/api/v1/segments/{id}` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 段落 ID |

### Body 参数（JSON，全部可选）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| text | string | 否 | 保持原值 | 段落文本，去除首尾空白后 1~2000 个字符 |
| seg_type | string | 否 | 保持原值 | 段落类型，只能是 `narration`（旁白）或 `dialogue`（对白） |
| speaker_id | integer | 否 | 保持原值 | 说话人角色 ID；传入后置信度置为 1.0（本接口不支持清空为 null） |
| voice_id | integer | 否 | 保持原值 | 段落指定音色 ID（本接口不支持清空为 null） |
| emotion | string | 否 | 保持原值 | 情绪标签，1~24 个字符 |
| rate_percent | integer | 否 | 保持原值 | 语速偏移（%），范围 -50~100 |
| pitch_hz | integer | 否 | 保持原值 | 音调偏移（Hz），范围 -50~50 |
| volume_percent | integer | 否 | 保持原值 | 音量偏移（%），范围 -50~50 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为更新后的段落（SegmentDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 段落 ID |
| index | integer | 段落序号 |
| text | string | 段落文本 |
| char_count | integer | 文本字符数 |
| seg_type | string | 段落类型：`narration` / `dialogue` |
| speaker_id | integer / null | 说话人角色 ID |
| speaker_confidence | float / null | 说话人置信度（手动指定后为 1.0） |
| voice_id | integer / null | 段落指定音色 ID |
| emotion | string | 情绪标签 |
| rate_percent | integer | 语速偏移（%） |
| pitch_hz | integer | 音调偏移（Hz） |
| volume_percent | integer | 音量偏移（%） |
| status | string | 合成状态，更新后固定为 `pending` |
| audio_duration_ms | integer / null | 段落音频时长，更新后为 `null` |
| error_message | string / null | 失败原因，更新后为 `null` |
| resolved_voice | object / null | 本接口响应中固定为 `null`（音色解析见段落列表接口） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 100,
    "index": 1,
    "text": "清晨的雾气还未散去，山门外传来一阵脚步声。",
    "char_count": 24,
    "seg_type": "narration",
    "speaker_id": null,
    "speaker_confidence": null,
    "voice_id": null,
    "emotion": "neutral",
    "rate_percent": 10,
    "pitch_hz": 0,
    "volume_percent": 0,
    "status": "pending",
    "audio_duration_ms": null,
    "error_message": null,
    "resolved_voice": null
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 文本为空 / 超 2000 字符；seg_type 非法；emotion 长度非法；rate/pitch/volume 超范围 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 段落不存在 |
| 409 | 20003 | 段落正在合成中，暂不能编辑 |

## curl 示例

```bash
curl -X PATCH 'http://localhost:60181/api/v1/segments/100' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "text": "清晨的雾气还未散去，山门外传来一阵脚步声。",
    "rate_percent": 10
  }'
```

为对白段落指定角色与音色：

```bash
curl -X PATCH 'http://localhost:60181/api/v1/segments/101' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "speaker_id": 3,
    "voice_id": 8,
    "emotion": "angry"
  }'
```