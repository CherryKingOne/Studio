# 书籍详情

获取单本书籍的完整信息，包括制作设置、封面、源文件、统计计数与状态。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/books/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（BookDetail）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 书籍 ID |
| title | string | 书名 |
| author | string / null | 作者 |
| description | string / null | 简介 |
| language | string | 语言，固定为 `zh-CN` |
| cover_file_id | integer / null | 封面文件 ID，未上传为 `null` |
| cover_url | string / null | 封面访问地址（`/api/v1/files/{id}`），未上传为 `null` |
| source_file_id | integer / null | 导入源 TXT 文件 ID，未导入为 `null` |
| default_narrator_voice_id | integer / null | 默认旁白音色 ID，未设置为 `null` |
| settings | object | 制作设置，结构见下表 |
| status | string | 书籍状态：`draft` / `importing` / `parsing` / `segmenting` / `ready` / `failed` / `archived` |
| total_chapters | integer | 章节总数 |
| total_segments | integer | 段落总数 |
| total_duration_ms | integer | 已拼接章节音频总时长（毫秒） |
| error_message | string / null | 最近一次失败原因，无错误为 `null` |
| created_at | string | 创建时间（ISO 8601 UTC） |
| updated_at | string | 更新时间（ISO 8601 UTC） |

`settings` 对象字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| pause_ms.narration | integer | 旁白段落后的停顿（毫秒），默认 350 |
| pause_ms.dialogue | integer | 对白段落后的停顿（毫秒），默认 250 |
| pause_ms.chapter_end | integer | 章节结尾停顿（毫秒），默认 1200 |
| loudness.i | float | 目标响度（LUFS），默认 -16.0 |
| loudness.tp | float | 真峰值上限（dBTP），默认 -1.5 |
| loudness.lra | float | 响度范围（LU），默认 11.0 |
| output.format | string | 输出格式，默认 `mp3` |
| output.sample_rate | integer | 采样率（Hz），默认 44100 |
| output.channels | integer | 声道数，默认 1 |
| output.bitrate_kbps | integer | 比特率（kbps），默认 128 |
| tts.provider | string | 默认合成供应商，默认 `openai` |
| tts.max_attempts | integer | 单段落最大重试次数，默认 3 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 1,
    "title": "示例小说",
    "author": "佚名",
    "description": "一本用于演示的小说",
    "language": "zh-CN",
    "cover_file_id": 12,
    "cover_url": "/api/v1/files/12",
    "source_file_id": 3,
    "default_narrator_voice_id": 5,
    "settings": {
      "pause_ms": { "narration": 350, "dialogue": 250, "chapter_end": 1200 },
      "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
      "output": { "format": "mp3", "sample_rate": 44100, "channels": 1, "bitrate_kbps": 128 },
      "tts": { "provider": "openai", "max_attempts": 3 }
    },
    "status": "ready",
    "total_chapters": 20,
    "total_segments": 480,
    "total_duration_ms": 3600000,
    "error_message": null,
    "created_at": "2026-11-13T08:00:00Z",
    "updated_at": "2026-11-13T09:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已软删除 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/books/1' \
  -H 'Authorization: Bearer <access_token>'
```