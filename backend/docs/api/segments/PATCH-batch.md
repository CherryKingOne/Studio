# 批量更新段落

一次性更新多个段落（1~200 个），常用于批量指派角色 / 音色、批量调整合成参数。每个元素为「段落 ID + 可选更新字段」，字段含义与单段落更新一致。

**副作用**：所有被更新段落状态重置为 `pending` 并清除段落音频；涉及的所有章节的整章拼接音频也会被清除。整个批量操作在一个事务中执行，任一段落校验失败则全部不生效。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `PATCH` |
| 请求路径 | `/api/v1/segments/batch` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| items | array | 是 | - | 更新项数组，长度 1~200；段落 ID 不能重复 |

`items[]` 元素字段：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| id | integer | 是 | - | 段落 ID |
| text | string | 否 | 保持原值 | 段落文本，1~2000 个字符 |
| seg_type | string | 否 | 保持原值 | `narration` / `dialogue` |
| speaker_id | integer | 否 | 保持原值 | 说话人角色 ID，传入后置信度置 1.0 |
| voice_id | integer | 否 | 保持原值 | 段落指定音色 ID |
| emotion | string | 否 | 保持原值 | 情绪标签，1~24 个字符 |
| rate_percent | integer | 否 | 保持原值 | 语速偏移（%），-50~100 |
| pitch_hz | integer | 否 | 保持原值 | 音调偏移（Hz），-50~50 |
| volume_percent | integer | 否 | 保持原值 | 音量偏移（%），-50~50 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| updated | integer | 成功更新的段落数量 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "updated": 3
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | items 为空 / 超过 200 个 / 存在重复段落 ID；任一字段校验失败 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 任一段落不存在 |
| 409 | 20003 | 任一段落正在合成中 |

## curl 示例

```bash
curl -X PATCH 'http://localhost:60181/api/v1/segments/batch' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "items": [
      { "id": 101, "speaker_id": 3, "voice_id": 8 },
      { "id": 102, "speaker_id": 3, "voice_id": 8 },
      { "id": 105, "rate_percent": -10, "emotion": "sad" }
    ]
  }'
```