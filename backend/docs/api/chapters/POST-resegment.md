# 重新切段

按指定最大字符数将章节正文重新切分为段落：**删除旧段落并重建**。用于调整段落粒度或正文编辑后重建段落。必须显式确认（`confirm=true`），且章节不能已有合成音频。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/chapters/{id}/resegment` |
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
| confirm | boolean | 是 | false | 必须显式传 `true` 确认重新切段（会删除现有段落），否则返回 `20002` |
| max_chars | integer | 否 | `IMPORT_MAX_CHARS`（默认 300） | 单段最大字符数，取值范围 50~1000；切段按句子边界就近切分 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| segments_created | integer | 新建段落数量 |
| char_count | integer | 章节正文字符数 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "segments_created": 12,
    "char_count": 3200
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | `confirm` 不为 `true`；`max_chars` 不在 50~1000 之间 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节不存在 |
| 409 | 20003 | 章节已有合成音频，不能重新切段；或正文无法切分出段落 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/chapters/10/resegment' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "confirm": true,
    "max_chars": 300
  }'
```

> 重新切段后段落状态均为 `pending`，章节状态置为 `segmented`，需重新发起 TTS 合成。