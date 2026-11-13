# 更新书籍

部分更新书籍信息。仅传入需要修改的字段，未传入的字段保持原值。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `PATCH` |
| 请求路径 | `/api/v1/books/{id}` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

### Body 参数（JSON，全部可选）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| title | string | 否 | 保持原值 | 书名，去除首尾空白后 1~255 个字符；改名后与其他书籍重名时返回 `20004` |
| author | string | 否 | 保持原值 | 作者，空字符串按 `null` 处理 |
| description | string | 否 | 保持原值 | 简介，空字符串按 `null` 处理 |
| default_narrator_voice_id | integer | 否 | 保持原值 | 默认旁白音色 ID；传入后覆盖原值（本接口不支持将其清空为 null） |
| settings | object | 否 | 保持原值 | 制作设置，**整体替换**。子对象缺省时使用代码默认值（而非书籍当前值），建议传完整结构 |

`settings` 对象字段（缺省子对象使用默认值）：

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| pause_ms.narration | integer | 350 | 旁白段落后的停顿（毫秒） |
| pause_ms.dialogue | integer | 250 | 对白段落后的停顿（毫秒） |
| pause_ms.chapter_end | integer | 1200 | 章节结尾停顿（毫秒） |
| loudness.i | float | -16.0 | 目标响度（LUFS） |
| loudness.tp | float | -1.5 | 真峰值上限（dBTP） |
| loudness.lra | float | 11.0 | 响度范围（LU） |
| output.format | string | mp3 | 输出格式 |
| output.sample_rate | integer | 44100 | 采样率（Hz） |
| output.channels | integer | 1 | 声道数 |
| output.bitrate_kbps | integer | 128 | 比特率（kbps） |
| tts.provider | string | openai | 默认合成供应商 |
| tts.max_attempts | integer | 3 | 单段落最大重试次数 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为更新后的完整书籍详情（BookDetail），字段与 [书籍详情](GET-detail.md) 一致：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 书籍 ID |
| title | string | 书名 |
| author | string / null | 作者 |
| description | string / null | 简介 |
| language | string | 语言，固定 `zh-CN` |
| cover_file_id | integer / null | 封面文件 ID |
| cover_url | string / null | 封面访问地址 |
| source_file_id | integer / null | 导入源文件 ID |
| default_narrator_voice_id | integer / null | 默认旁白音色 ID |
| settings | object | 制作设置（结构见请求参数） |
| status | string | 书籍状态 |
| total_chapters | integer | 章节总数 |
| total_segments | integer | 段落总数 |
| total_duration_ms | integer | 音频总时长（毫秒） |
| error_message | string / null | 最近一次失败原因 |
| created_at | string | 创建时间（ISO 8601 UTC） |
| updated_at | string | 更新时间（ISO 8601 UTC） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 1,
    "title": "示例小说（修订版）",
    "author": "佚名",
    "description": "一本用于演示的小说",
    "language": "zh-CN",
    "cover_file_id": 12,
    "cover_url": "/api/v1/files/12",
    "source_file_id": 3,
    "default_narrator_voice_id": 5,
    "settings": {
      "pause_ms": { "narration": 400, "dialogue": 250, "chapter_end": 1200 },
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
    "updated_at": "2026-11-13T10:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 书名为空或超过 255 个字符 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已软删除 |
| 409 | 20004 | 改名后与其他书籍重名 |

## curl 示例

```bash
curl -X PATCH 'http://localhost:60181/api/v1/books/1' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "title": "示例小说（修订版）",
    "default_narrator_voice_id": 5,
    "settings": {
      "pause_ms": { "narration": 400, "dialogue": 250, "chapter_end": 1200 },
      "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
      "output": { "format": "mp3", "sample_rate": 44100, "channels": 1, "bitrate_kbps": 128 },
      "tts": { "provider": "openai", "max_attempts": 3 }
    }
  }'
```