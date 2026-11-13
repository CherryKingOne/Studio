# 章节列表

分页查询某本书籍的章节列表，按章节序号升序排列，支持按状态、标题关键词筛选。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/books/{book_id}/chapters` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| book_id | integer | 是 | 书籍 ID |

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| page | integer | 否 | 1 | 页码，从 1 开始，小于 1 按 1 处理 |
| page_size | integer | 否 | 20 | 每页条数，限制 1~100 |
| status | string | 否 | - | 按状态精确筛选：`pending` / `segmented` / `tts_partial` / `tts_done` / `merged` |
| keyword | string | 否 | - | 关键词，对章节标题做模糊匹配（LIKE %keyword%） |

## 响应参数

HTTP `200`，统一包装结构，`data` 为分页对象：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 章节摘要列表，元素结构见下表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 满足条件的章节总数 |

`items[]` 元素字段（ChapterSummary）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 章节 ID |
| index | integer | 章节序号（从 1 开始，对应数据库 idx） |
| title | string | 章节标题 |
| char_count | integer | 正文字符数 |
| status | string | 章节状态：`pending`（待切段）/ `segmented`（已切段）/ `tts_partial`（部分合成）/ `tts_done`（全部合成）/ `merged`（已拼接） |
| audio_duration_ms | integer / null | 整章音频时长（毫秒），未拼接为 `null` |
| tts_done_segments | integer | 已合成完成的段落数 |
| segment_count | integer | 段落总数 |
| error_message | string / null | 最近一次失败原因，无错误为 `null` |
| updated_at | string | 更新时间（ISO 8601 UTC） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 10,
        "index": 1,
        "title": "第一章 初入江湖",
        "char_count": 3200,
        "status": "merged",
        "audio_duration_ms": 420000,
        "tts_done_segments": 12,
        "segment_count": 12,
        "error_message": null,
        "updated_at": "2026-11-13T09:00:00Z"
      }
    ],
    "page": 1,
    "page_size": 20,
    "total": 20
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

> 书籍 ID 不存在时返回空列表（`items: []`、`total: 0`），不返回 404。

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/books/1/chapters?page=1&page_size=20' \
  -H 'Authorization: Bearer <access_token>'
```

按状态筛选：

```bash
curl -X GET 'http://localhost:60181/api/v1/books/1/chapters?status=tts_partial&keyword=第一章' \
  -H 'Authorization: Bearer <access_token>'
```