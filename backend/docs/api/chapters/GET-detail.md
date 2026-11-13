# 章节详情

获取单个章节的完整信息，包括正文全文、合成与拼接状态、音频文件关联。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/chapters/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 章节 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（ChapterDetail）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 章节 ID |
| book_id | integer | 所属书籍 ID |
| index | integer | 章节序号（从 1 开始） |
| title | string | 章节标题 |
| raw_text | string | 章节正文全文 |
| char_count | integer | 正文字符数 |
| status | string | 章节状态：`pending` / `segmented` / `tts_partial` / `tts_done` / `merged` |
| narrator_voice_id | integer / null | 章节级旁白音色 ID（优先于书籍默认旁白），未设置为 `null` |
| audio_file_id | integer / null | 整章拼接音频文件 ID，未拼接为 `null` |
| audio_duration_ms | integer / null | 整章音频时长（毫秒），未拼接为 `null` |
| audio_rev | integer | 音频版本号，每次重新拼接递增，用于缓存失效 |
| tts_done_segments | integer | 已合成完成的段落数 |
| error_message | string / null | 最近一次失败原因，无错误为 `null` |
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
    "title": "第一章 初入江湖",
    "raw_text": "清晨的雾气还未散去……",
    "char_count": 3200,
    "status": "merged",
    "narrator_voice_id": null,
    "audio_file_id": 56,
    "audio_duration_ms": 420000,
    "audio_rev": 2,
    "tts_done_segments": 12,
    "error_message": null,
    "created_at": "2026-11-13T08:10:00Z",
    "updated_at": "2026-11-13T09:00:00Z"
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
curl -X GET 'http://localhost:60181/api/v1/chapters/10' \
  -H 'Authorization: Bearer <access_token>'
```