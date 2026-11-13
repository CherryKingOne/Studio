# 创建书籍

手动创建一本书籍（不含正文导入）。书名必填且全局唯一，创建后状态为 `draft`，并写入默认制作设置。导入正文请使用 `POST /api/v1/books/import`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| title | string | 是 | - | 书名，去除首尾空白后 1~255 个字符；同名书籍已存在时返回 `20004` |
| author | string | 否 | null | 作者，去除首尾空白，空字符串按 `null` 处理 |
| description | string | 否 | null | 简介，去除首尾空白，空字符串按 `null` 处理 |

## 响应参数

HTTP `201 Created`，统一包装结构（`message` 为 `created`），`data` 字段（BookDetail）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 书籍 ID |
| title | string | 书名 |
| author | string / null | 作者 |
| description | string / null | 简介 |
| language | string | 语言，固定为 `zh-CN` |
| cover_file_id | integer / null | 封面文件 ID，未上传为 `null` |
| cover_url | string / null | 封面访问地址（`/api/v1/files/{id}`），未上传为 `null` |
| source_file_id | integer / null | 导入源文件 ID，手动创建为 `null` |
| default_narrator_voice_id | integer / null | 默认旁白音色 ID，未设置为 `null` |
| settings | object | 制作设置，结构见下表 |
| status | string | 书籍状态，新建为 `draft` |
| total_chapters | integer | 章节总数，新建为 0 |
| total_segments | integer | 段落总数，新建为 0 |
| total_duration_ms | integer | 音频总时长（毫秒），新建为 0 |
| error_message | string / null | 最近一次失败原因，无错误为 `null` |
| created_at | string | 创建时间（ISO 8601 UTC） |
| updated_at | string | 更新时间（ISO 8601 UTC） |

`settings` 对象字段（创建时写入的默认值）：

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| pause_ms | object | - | 拼接停顿时长（毫秒） |
| pause_ms.narration | integer | 350 | 旁白段落后的停顿 |
| pause_ms.dialogue | integer | 250 | 对白段落后的停顿 |
| pause_ms.chapter_end | integer | 1200 | 章节结尾停顿 |
| loudness | object | - | 响度归一化参数（EBU R128） |
| loudness.i | float | -16.0 | 目标 integrated loudness（LUFS） |
| loudness.tp | float | -1.5 | 真峰值上限（dBTP） |
| loudness.lra | float | 11.0 | 响度范围（LU） |
| output | object | - | 输出音频参数 |
| output.format | string | mp3 | 输出格式 |
| output.sample_rate | integer | 44100 | 采样率（Hz） |
| output.channels | integer | 1 | 声道数（1 单声道 / 2 立体声） |
| output.bitrate_kbps | integer | 128 | 比特率（kbps） |
| tts | object | - | TTS 合成设置 |
| tts.provider | string | openai | 默认合成供应商 |
| tts.max_attempts | integer | 3 | 单段落最大重试次数 |

## 响应示例

```json
{
  "code": 0,
  "message": "created",
  "data": {
    "id": 1,
    "title": "示例小说",
    "author": "佚名",
    "description": "一本用于演示的小说",
    "language": "zh-CN",
    "cover_file_id": null,
    "cover_url": null,
    "source_file_id": null,
    "default_narrator_voice_id": null,
    "settings": {
      "pause_ms": { "narration": 350, "dialogue": 250, "chapter_end": 1200 },
      "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
      "output": { "format": "mp3", "sample_rate": 44100, "channels": 1, "bitrate_kbps": 128 },
      "tts": { "provider": "openai", "max_attempts": 3 }
    },
    "status": "draft",
    "total_chapters": 0,
    "total_segments": 0,
    "total_duration_ms": 0,
    "error_message": null,
    "created_at": "2026-11-13T08:00:00Z",
    "updated_at": "2026-11-13T08:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 书名为空或超过 255 个字符 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 409 | 20004 | 同名书籍已存在 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/books' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "示例小说",
    "author": "佚名",
    "description": "一本用于演示的小说"
  }'
```