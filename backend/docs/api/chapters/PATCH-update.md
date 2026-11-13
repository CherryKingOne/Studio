# 更新章节

部分更新章节标题或正文。仅传入需要修改的字段，未传入的字段保持原值。

**状态约束：**
- 修改正文（`raw_text`）时，若章节已有拼接音频或存在已合成完成的段落，返回 `409`（20003）。
- 修改标题会清除章节已拼接的音频（需要重新拼接）。
- 修改正文后章节状态置为 `segmented` 并重新统计字数；已有段落不会自动重建，如需按新正文重新切段请调用 [重新切段](POST-resegment.md)。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `PATCH` |
| 请求路径 | `/api/v1/chapters/{id}` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 章节 ID |

### Body 参数（JSON，全部可选）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| title | string | 否 | 保持原值 | 章节标题，去除首尾空白后 1~255 个字符 |
| raw_text | string | 否 | 保持原值 | 章节正文，去除首尾空白后不能为空；章节已有合成音频时禁止修改 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为更新后的完整章节详情（ChapterDetail），字段与 [章节详情](GET-detail.md) 一致：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 章节 ID |
| book_id | integer | 所属书籍 ID |
| index | integer | 章节序号 |
| title | string | 章节标题 |
| raw_text | string | 章节正文全文 |
| char_count | integer | 正文字符数 |
| status | string | 章节状态 |
| narrator_voice_id | integer / null | 章节级旁白音色 ID |
| audio_file_id | integer / null | 整章音频文件 ID（改标题后会被清空） |
| audio_duration_ms | integer / null | 整章音频时长（毫秒） |
| audio_rev | integer | 音频版本号 |
| tts_done_segments | integer | 已合成完成的段落数 |
| error_message | string / null | 最近一次失败原因 |
| created_at | string | 创建时间（ISO 8601 UTC） |
| updated_at | string | 更新时间（ISO 8601 UTC） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 10,
    "book_id": 1,
    "index": 1,
    "title": "第一章 初入江湖（修订）",
    "raw_text": "清晨的雾气还未散去……",
    "char_count": 3200,
    "status": "segmented",
    "narrator_voice_id": null,
    "audio_file_id": null,
    "audio_duration_ms": null,
    "audio_rev": 2,
    "tts_done_segments": 0,
    "error_message": null,
    "created_at": "2026-11-13T08:10:00Z",
    "updated_at": "2026-11-13T10:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 标题长度不在 1~255 之间，或正文为空 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节不存在 |
| 409 | 20003 | 章节已有合成音频，不能直接修改正文 |

## curl 示例

仅修改标题：

```bash
curl -X PATCH 'http://localhost:60181/api/v1/chapters/10' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"title": "第一章 初入江湖（修订）"}'
```

修改正文：

```bash
curl -X PATCH 'http://localhost:60181/api/v1/chapters/10' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"raw_text": "清晨的雾气还未散去，少年背起行囊走出了山门。"}'
```