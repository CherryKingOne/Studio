# 章节音频拼接

将章节下已合成（`done`）的段落音频按顺序拼接为整章 MP3，插入停顿、做响度归一化（FFmpeg loudnorm），并写入 ID3 标签。异步任务（`chapter_merge`），接口立即返回 `202`。

**前置条件**：章节至少有一个已合成段落；存在未完成段落时必须设置 `skip_failed=true`，否则返回 `409`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/chapters/{id}/merge` |
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
| skip_failed | boolean | 否 | false | 是否跳过未完成段落继续拼接；为 `false` 且存在未完成段落时返回 `20003` |
| pause_ms | object | 否 | 书籍设置 `settings.pause_ms` | 停顿时长（毫秒），三个子字段必须同时提供 |
| pause_ms.narration | integer | 条件必填 | 350 | 旁白段落后的停顿，0~5000 |
| pause_ms.dialogue | integer | 条件必填 | 250 | 对白段落后的停顿，0~5000 |
| pause_ms.chapter_end | integer | 条件必填 | 1200 | 章节结尾停顿，0~10000 |
| loudness | object | 否 | 书籍设置 `settings.loudness` | 响度归一化参数，三个子字段必须同时提供 |
| loudness.i | float | 条件必填 | -16.0 | 目标响度（LUFS），范围 -40~-5 |
| loudness.tp | float | 条件必填 | -1.5 | 真峰值上限（dBTP），范围 -9~0 |
| loudness.lra | float | 条件必填 | 11.0 | 响度范围（LU），范围 1~50 |
| output | object | 否 | 书籍设置 `settings.output` | 输出参数，三个子字段必须同时提供 |
| output.sample_rate | integer | 条件必填 | 44100 | 采样率（Hz），仅支持 24000 / 44100 / 48000 |
| output.channels | integer | 条件必填 | 1 | 声道数，仅支持 1（单声道）/ 2（立体声） |
| output.bitrate_kbps | integer | 条件必填 | 128 | 比特率（kbps），范围 64~320 |

> 「条件必填」指：一旦提供该对象，其全部子字段都必须提供（对象整体缺省时使用书籍设置）。

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| job_id | integer | 拼接任务 ID（`job_type` 为 `chapter_merge`） |

任务完成后，任务的 `result` 字段包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| chapter_id | integer | 章节 ID |
| file_id | integer | 生成的整章音频文件 ID |
| duration_ms | integer | 整章音频时长（毫秒） |
| skipped_segments | integer | 实际参与拼接的段落数（字段名为历史遗留，含义为已拼接段落数） |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "job_id": 305
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 停顿 / 响度 / 输出参数超出范围 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节或所属书籍不存在 |
| 409 | 20003 | 章节没有已合成的段落；或存在未完成段落且未设置 skip_failed=true |
| 409 | 40004 | 该章节已有进行中的拼接任务，`details.existing_job_id` 为进行中任务 ID |

## curl 示例

使用书籍默认设置拼接：

```bash
curl -X POST 'http://localhost:60181/api/v1/chapters/10/merge' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"skip_failed": false}'
```

自定义停顿与输出参数，跳过失败段落：

```bash
curl -X POST 'http://localhost:60181/api/v1/chapters/10/merge' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "skip_failed": true,
    "pause_ms": { "narration": 400, "dialogue": 250, "chapter_end": 1500 },
    "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
    "output": { "sample_rate": 44100, "channels": 1, "bitrate_kbps": 192 }
  }'
```

> 拼接成功后章节状态置为 `merged`，`audio_rev` 递增；可通过 `GET /api/v1/chapters/{id}/audio` 下载。