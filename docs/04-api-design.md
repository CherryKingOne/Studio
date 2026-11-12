# 04 API 接口设计

> 文档版本：v0.1（评审稿）
> Base URL：`/api/v1`
> 协议：HTTP/JSON（除音频、ZIP、SSE 外）
> 认证：`Authorization: Bearer <JWT>`
> OpenAPI：开发环境 `GET /docs`（Swagger UI），`GET /api-docs/openapi.json`

## 1. 通用规范

### 1.1 请求与响应

- 请求/响应 JSON 字段一律 `snake_case`。
- 时间格式：ISO-8601 UTC，例如 `2026-11-12T08:00:00Z`。
- ID：JSON number（BIGINT 范围内）。
- 字符编码：UTF-8。
- 除登录、健康检查外，所有接口要求 Bearer Token。

### 1.2 统一响应包络

成功：

```json
{
  "code": 0,
  "message": "ok",
  "data": {},
  "request_id": "0193f0c4-8f7a-7c31-9b2e-2f1d3a4b5c6d"
}
```

失败：

```json
{
  "code": 20002,
  "message": "参数校验失败",
  "details": [
    { "field": "title", "message": "不能为空" }
  ],
  "request_id": "0193f0c4-8f7a-7c31-9b2e-2f1d3a4b5c6d"
}
```

- `request_id` 由中间件生成（UUID v7），响应头同时返回 `X-Request-Id`。
- `details` 仅在参数校验/冲突等场景出现。

### 1.3 分页

请求：`?page=1&page_size=20`（`page` 从 1 开始，`page_size` 1–100，默认 20）。

响应 `data`：

```json
{
  "items": [],
  "page": 1,
  "page_size": 20,
  "total": 137
}
```

### 1.4 HTTP 状态码与业务码

| HTTP | 使用场景 |
| --- | --- |
| `200` | 查询、更新成功 |
| `201` | 同步创建资源成功（书籍、角色） |
| `202` | 已接受异步任务（导入、合成、拼接、导出、识别） |
| `204` | 删除/失效成功，无响应体 |
| `400` | 参数错误、状态不满足前置条件 |
| `401` | 未认证或 Token 失效 |
| `403` | 无权限（MVP 仅单管理员，基本不触发） |
| `404` | 资源不存在 |
| `409` | 状态冲突、重复任务、资源被占用 |
| `413` | 文件过大 |
| `415` | 文件类型不支持 |
| `429` | 限流 |
| `500` | 内部错误 |
| `502` | 外部服务（TTS/Storage）不可用（多用于同步试听） |

### 1.5 业务错误码

| code | 名称 | HTTP | 说明 |
| --- | --- | --- | --- |
| `0` | OK | 200/201/202/204 | 成功 |
| `10001` | `UNAUTHORIZED` | 401 | 未登录或 Token 无效 |
| `10002` | `FORBIDDEN` | 403 | 无权限 |
| `10003` | `LOGIN_FAILED` | 401 | 用户名或密码错误 |
| `10004` | `RATE_LIMITED` | 429 | 请求过于频繁 |
| `20001` | `NOT_FOUND` | 404 | 资源不存在或已删除 |
| `20002` | `VALIDATION_FAILED` | 400 | 参数校验失败 |
| `20003` | `INVALID_STATE` | 409 | 当前状态不允许该操作 |
| `20004` | `DUPLICATE_RESOURCE` | 409 | 名称等唯一约束冲突 |
| `20005` | `RESOURCE_IN_USE` | 409 | 资源被引用，不能删除 |
| `30001` | `UNSUPPORTED_FILE_TYPE` | 415 | 仅支持 .txt / 指定图片格式 |
| `30002` | `FILE_TOO_LARGE` | 413 | 超过上传限制 |
| `30003` | `STORAGE_ERROR` | 502 | 对象存储读写失败 |
| `40001` | `TTS_PROVIDER_ERROR` | 502 | Provider 调用失败（同步试听） |
| `40002` | `TTS_RATE_LIMITED` | 429 | Provider 限流（同步试听） |
| `40003` | `AUDIO_PROCESS_ERROR` | 500 | FFmpeg 处理失败 |
| `40004` | `JOB_ALREADY_ACTIVE` | 409 | 同一目标存在进行中的同类任务 |
| `50001` | `INTERNAL_ERROR` | 500 | 未分类内部错误 |

### 1.6 异步任务规则

| 规则 | 说明 |
| --- | --- |
| 创建任务 | 返回 `202`，`data` 含 `job_id`，客户端通过 SSE 或轮询获取进度 |
| 重复任务 | 同目标（同一章节/书籍）+ 同类型存在 `queued/running` 任务时，返回 `409/40004`，`data.existing_job_id` 给出已有任务 |
| 任务取消 | `queued` 直接取消；`running` 置 `cancel_requested=1`，Worker 在每个子项前检查 |
| 任务重试 | 对 `failed/partial` 创建新任务，保留原任务审计记录 |
| 任务进度 | `progress` 0–100；`done_items/total_items` 为子项计数 |

## 2. 接口总览

### 2.1 认证

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| POST | `/auth/login` | 登录获取 JWT | 200 |
| GET | `/auth/me` | 当前用户信息 | 200 |
| POST | `/auth/password` | 修改密码 | 204 |

### 2.2 书籍

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/books` | 书籍分页列表 | 200 |
| POST | `/books` | 手动创建书籍 | 201 |
| GET | `/books/{id}` | 书籍详情 | 200 |
| PATCH | `/books/{id}` | 更新书籍 | 200 |
| DELETE | `/books/{id}` | 软删除书籍 | 204 |
| POST | `/books/import` | 上传 TXT 并创建导入任务 | 202 |
| POST | `/books/{id}/cover` | 上传封面 | 200 |
| GET | `/books/{id}/stats` | 书籍统计 | 200 |

### 2.3 章节

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/books/{book_id}/chapters` | 章节分页列表 | 200 |
| GET | `/chapters/{id}` | 章节详情（含正文） | 200 |
| PATCH | `/chapters/{id}` | 编辑标题/正文 | 200 |
| POST | `/chapters/{id}/resegment` | 重新切段（同步） | 200 |
| POST | `/chapters/{id}/tts` | 创建整章合成任务 | 202 |
| POST | `/chapters/{id}/merge` | 创建章节拼接任务 | 202 |
| GET | `/chapters/{id}/audio` | 下载章节音频 | 200 |
| DELETE | `/chapters/{id}/audio` | 失效章节音频 | 204 |

### 2.4 段落

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/chapters/{chapter_id}/segments` | 段落列表（不分页，上限 2000） | 200 |
| PATCH | `/segments/{id}` | 编辑单段 | 200 |
| PATCH | `/segments/batch` | 批量编辑（≤200 条） | 200 |
| POST | `/segments/{id}/tts` | 单段合成任务 | 202 |
| GET | `/segments/{id}/audio` | 下载段落音频 | 200 |

### 2.5 角色

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/books/{book_id}/characters` | 角色列表 | 200 |
| POST | `/books/{book_id}/characters` | 创建角色 | 201 |
| PATCH | `/characters/{id}` | 更新角色（含绑定音色） | 200 |
| DELETE | `/characters/{id}` | 删除角色 | 204 |
| POST | `/books/{book_id}/characters/detect` | 创建角色识别任务 | 202 |

### 2.6 音色

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/voices` | 音色列表 | 200 |
| POST | `/voices/sync` | 同步 Provider 音色目录 | 200 |
| POST | `/voices/preview` | 文本试听（同步合成） | 200 |
| GET | `/voices/{id}/sample` | 获取音色样例音频 | 200 |

### 2.7 任务

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/jobs` | 任务分页列表 | 200 |
| GET | `/jobs/{id}` | 任务详情 | 200 |
| POST | `/jobs/{id}/cancel` | 取消任务 | 200 |
| POST | `/jobs/{id}/retry` | 重试任务（创建新任务） | 202 |
| GET | `/jobs/{id}/events` | SSE 进度流 | 200 |

### 2.8 导出与文件

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| POST | `/books/{book_id}/exports` | 创建整书导出任务 | 202 |
| GET | `/exports/{id}` | 导出记录详情 | 200 |
| GET | `/exports/{id}/download` | 下载 ZIP | 200 |
| GET | `/files/{id}` | 鉴权下载文件 | 200 |

### 2.9 健康检查

> 健康检查挂载在根路径，不使用 `/api/v1` 前缀。

| 方法 | 路径 | 说明 | 成功码 |
| --- | --- | --- | --- |
| GET | `/healthz` | 存活检查 | 200 |
| GET | `/readyz` | 依赖就绪检查 | 200/503 |

## 3. 接口详情

### 3.1 认证

#### POST `/auth/login`

请求：

```json
{ "username": "admin", "password": "your-password" }
```

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "access_token": "eyJhbGciOi...",
    "token_type": "Bearer",
    "expires_in": 86400,
    "user": { "id": 1, "username": "admin", "display_name": "管理员" }
  },
  "request_id": "..."
}
```

错误：`10003` 用户名或密码错误；`10004` 登录限流（10 次/分钟/IP）。

#### GET `/auth/me`

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": { "id": 1, "username": "admin", "display_name": "管理员", "role": "admin", "last_login_at": "2026-11-12T08:00:00Z" },
  "request_id": "..."
}
```

#### POST `/auth/password`

请求：

```json
{ "old_password": "old", "new_password": "new-with-at-least-8-chars" }
```

响应 `204`。错误：`10003` 旧密码错误；`20002` 新密码长度不足。

### 3.2 书籍

#### GET `/books`

查询参数：`page`、`page_size`、`keyword`（匹配书名/作者）、`status`、`include_archived`（默认 false）。

响应 `200` 的 `data`：

```json
{
  "items": [
    {
      "id": 12,
      "title": "山海拾遗",
      "author": "某某",
      "cover_url": "/api/v1/files/88",
      "status": "ready",
      "total_chapters": 36,
      "total_segments": 842,
      "total_duration_ms": 5400000,
      "created_at": "2026-11-12T08:00:00Z",
      "updated_at": "2026-11-12T09:30:00Z"
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

说明：`cover_url` 为后端代理下载路径，前端直接请求（需带 Token）。

#### POST `/books`

请求：

```json
{ "title": "山海拾遗", "author": "某某", "description": "简介" }
```

响应 `201`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 12,
    "title": "山海拾遗",
    "author": "某某",
    "status": "draft",
    "settings": {
      "pause_ms": { "narration": 350, "dialogue": 250, "chapter_end": 1200 },
      "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
      "output": { "format": "mp3", "sample_rate": 44100, "channels": 1, "bitrate_kbps": 128 }
    },
    "created_at": "2026-11-12T08:00:00Z"
  },
  "request_id": "..."
}
```

错误：`20004` 同名书籍已存在（同一未删除范围）。

#### GET `/books/{id}`

响应 `200`：书籍完整信息 + `default_narrator_voice_id`、`source_file_id`、`error_message`、`settings`。

#### PATCH `/books/{id}`

请求（字段均可选）：`title`、`author`、`description`、`default_narrator_voice_id`、`settings`（整体替换，服务端校验范围）。

响应 `200`：更新后的书籍。错误：`404`；`20002`；`20003`（导入进行中禁止修改 `settings`）。

#### DELETE `/books/{id}`

响应 `204`。软删除，不删除对象存储文件与任务记录。错误：`20003`（存在进行中任务）。

#### POST `/books/import`

`Content-Type: multipart/form-data`。

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `file` | 是 | .txt，≤50MB，UTF-8/GBK/GB18030 |
| `title` | 否 | 缺省取文件名（去扩展名） |
| `author` | 否 | 作者 |

响应 `202`：

```json
{
  "code": 0,
  "message": "accepted",
  "data": { "book_id": 13, "job_id": 501 },
  "request_id": "..."
}
```

错误：`30001` 非 txt；`30002` 超过 50MB；`30003` 上传失败。

#### POST `/books/{id}/cover`

`multipart/form-data`，字段 `file`：jpg/png/webp ≤5MB。

响应 `200`：

```json
{ "code": 0, "message": "ok", "data": { "cover_file_id": 88, "cover_url": "/api/v1/files/88" }, "request_id": "..." }
```

#### GET `/books/{id}/stats`

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "total_chapters": 36,
    "segmented_chapters": 36,
    "tts_done_chapters": 30,
    "merged_chapters": 30,
    "total_segments": 842,
    "done_segments": 700,
    "failed_segments": 3,
    "total_duration_ms": 5400000,
    "active_job_count": 1
  },
  "request_id": "..."
}
```

### 3.3 章节

#### GET `/books/{book_id}/chapters`

查询：`page`、`page_size`、`status`、`keyword`（标题）。

响应 `200` 的 `data.items[]`：

```json
{
  "id": 101,
  "index": 1,
  "title": "第一章 山雨",
  "char_count": 3200,
  "status": "tts_done",
  "narrator_voice_id": null,
  "audio_duration_ms": 480000,
  "tts_done_segments": 42,
  "segment_count": 42,
  "error_message": null,
  "updated_at": "2026-11-12T09:00:00Z"
}
```

#### GET `/chapters/{id}`

响应 `200`：章节全部字段，含 `raw_text`（正文）。

#### PATCH `/chapters/{id}`

请求：

```json
{ "title": "第一章 山雨（校对）", "raw_text": "……" }
```

规则：

1. `title` 任意状态可改。
2. `raw_text` 仅允许在章节无任何已完成音频时修改（`audio_file_id` 为空且无 `done` 段落），否则 `409/20003`。
3. 修改 `raw_text` 后，`status` 置为 `segmented`（需重新切段），`char_count` 重算。

响应 `200`：更新后的章节。

#### POST `/chapters/{id}/resegment`

同步执行切段。请求：

```json
{ "confirm": true, "max_chars": 300 }
```

前置条件：

1. `audio_file_id` 为空。
2. 不存在 `done` 状态的段落。
3. `confirm=true`（确认将删除旧段落）。

响应 `200`：

```json
{ "code": 0, "message": "ok", "data": { "segments_created": 42, "char_count": 3200 }, "request_id": "..." }
```

错误：`20003` 已有音频或缺少 confirm；`20002` `max_chars` 超出 50–1000。

#### POST `/chapters/{id}/tts`

请求：

```json
{ "mode": "missing", "force": false }
```

| 字段 | 说明 |
| --- | --- |
| `mode` | `missing`（pending+failed，默认）/ `failed` / `all` |
| `force` | `true` 时忽略缓存并重新合成；默认 false |

响应 `202`：

```json
{ "code": 0, "message": "accepted", "data": { "job_id": 601, "total_candidates": 42, "cached_hits": 5 }, "request_id": "..." }
```

错误：`20003`（章节未切段）；`40004`（已有进行中合成任务，`data.existing_job_id`）；`20002`（mode 非法）。

#### POST `/chapters/{id}/merge`

请求：

```json
{
  "skip_failed": false,
  "pause_ms": { "narration": 350, "dialogue": 250, "chapter_end": 1200 },
  "loudness": { "i": -16.0, "tp": -1.5, "lra": 11.0 },
  "output": { "format": "mp3", "sample_rate": 44100, "channels": 1, "bitrate_kbps": 128 }
}
```

字段均可选，缺省使用 `books.settings`。

响应 `202`：`{ "job_id": 701 }`。

错误：`20003` 存在未完成段落且 `skip_failed=false`；`20003` 无任何已完成段落；`40004` 重复任务。

#### GET `/chapters/{id}/audio`

响应 `200`：`Content-Type: audio/mpeg`，`Content-Disposition: inline; filename="001-第一章-山雨.mp3"`。

错误：`20001` 音频不存在（未拼接）。

#### DELETE `/chapters/{id}/audio`

响应 `204`。将 `audio_file_id` 置空、`audio_rev+1`、章节状态回退为 `tts_done`（全部段落完成）或 `tts_partial`。

### 3.4 段落

#### GET `/chapters/{chapter_id}/segments`

查询：`seg_type`、`status`、`speaker_id`。返回数组（不包分页），上限 2000 条。

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 9001,
        "index": 1,
        "text": "夜色像一层薄薄的墨。",
        "seg_type": "narration",
        "speaker_id": null,
        "speaker_name": null,
        "speaker_confidence": null,
        "voice_id": null,
        "resolved_voice": { "voice_id": 3, "provider": "edge", "provider_voice_id": "zh-CN-XiaoxiaoNeural", "display_name": "晓晓" },
        "emotion": "neutral",
        "rate_percent": 0,
        "pitch_hz": 0,
        "volume_percent": 0,
        "status": "done",
        "audio_duration_ms": 8200,
        "error_message": null
      }
    ],
    "total": 42
  },
  "request_id": "..."
}
```

`resolved_voice` 为服务端按优先级计算出的实际音色，便于前端展示。

#### PATCH `/segments/{id}`

请求（均可选）：`text`、`seg_type`、`speaker_id`、`voice_id`、`emotion`、`rate_percent`、`pitch_hz`、`volume_percent`。

* 传 `speaker_id: null` 表示解除说话人；传 `voice_id: null` 表示取消段落级覆盖。

响应 `200`：更新后的段落。

副作用：

1. `text` 或音色参数变化 → `status=pending`，清空 `audio_file_id/audio_duration_ms/cache_key`，`attempts=0`。
2. 章节 `audio_file_id` 失效（置空），章节状态回退。
3. `seg_type` 从对白改为旁白且 `voice_id/speaker_id` 为空时，使用旁白音色。
4. 修改 `speaker_id` 时，`speaker_confidence` 置为 `1.000`（视为人工确认，后续自动识别不覆盖）。

错误：`20003`（段落正在合成中）；`20002`（`rate_percent` 超出 -50..100 等）。

#### PATCH `/segments/batch`

请求：

```json
{
  "items": [
    { "id": 9001, "speaker_id": 5 },
    { "id": 9002, "speaker_id": 6, "voice_id": 12 }
  ]
}
```

限制：≤200 条；事务内全部成功或全部失败。

响应 `200`：`{ "updated": 2 }`。

#### POST `/segments/{id}/tts`

请求：

```json
{ "force": false }
```

响应 `202`：`{ "job_id": 602, "cached_hit": false }`。

规则：若缓存命中且未强制，仍创建任务但 Worker 会直接关联缓存文件，子项标记 `done`。

#### GET `/segments/{id}/audio`

响应 `200`：`audio/mpeg`。错误：`20001`。

### 3.5 角色

#### GET `/books/{book_id}/characters`

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 5,
        "name": "阿澜",
        "aliases": ["澜儿", "阿澜姑娘"],
        "gender": "female",
        "age_group": "adult",
        "description": "山中的药师",
        "default_voice_id": 8,
        "color": "#7C9CBF",
        "source": "rule",
        "segment_count": 63
      }
    ],
    "total": 8
  },
  "request_id": "..."
}
```

#### POST `/books/{book_id}/characters`

请求：

```json
{ "name": "阿澜", "aliases": ["澜儿"], "gender": "female", "age_group": "adult", "description": "山中的药师", "default_voice_id": 8, "color": "#7C9CBF" }
```

响应 `201`。错误：`20004`（同书重名）。

#### PATCH `/characters/{id}`

可更新上述所有字段，另加 `default_voice_id`（绑定/解绑音色）。

响应 `200`。副作用：若修改了音色，引用该角色且无段落级覆盖的 `done` 段落置为 `pending`，受影响的章节音频失效。

#### DELETE `/characters/{id}`

响应 `204`。`segments.speaker_id` 置空（外键 `SET NULL`）；不删除段落。

#### POST `/books/{book_id}/characters/detect`

请求：

```json
{ "overwrite_existing": false }
```

响应 `202`：`{ "job_id": 801 }`。

规则：识别并创建/合并角色，回填 `segments.speaker_id`；`overwrite_existing=false` 时只填充 `speaker_id` 为空的段落。

### 3.6 音色

#### GET `/voices`

查询：`provider`、`locale`、`gender`、`enabled`、`keyword`、`page`、`page_size`。

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      { "id": 3, "provider": "edge", "provider_voice_id": "zh-CN-XiaoxiaoNeural", "display_name": "晓晓", "locale": "zh-CN", "gender": "female", "styles": ["gentle"], "enabled": true }
    ],
    "page": 1,
    "page_size": 50,
    "total": 21
  },
  "request_id": "..."
}
```

#### POST `/voices/sync`

请求：

```json
{ "provider": "edge" }
```

响应 `200`：`{ "synced": 322, "provider": "edge" }`。

错误：`40001` Provider 不可用；`20002` Provider 未实现。

#### POST `/voices/preview`

请求：

```json
{ "voice_id": 3, "text": "你好，这是一段音色试听。", "emotion": "neutral", "rate_percent": 0, "pitch_hz": 0, "volume_percent": 0 }
```

限制：`text` ≤100 字；限流 20 次/分钟。

响应 `200`：`audio/mpeg`（`Content-Type: audio/mpeg`，`Cache-Control: no-store`）。

错误：`40001` 合成失败；`40002` Provider 限流。

#### GET `/voices/{id}/sample`

响应 `200`：`audio/mpeg`。若样例不存在则同步生成并缓存（`files.kind=voice_sample`）。

### 3.7 任务

#### GET `/jobs`

查询：`page`、`page_size`、`book_id`、`job_type`、`status`。

响应 `200` 的 `data.items[]`：

```json
{
  "id": 601,
  "job_type": "chapter_tts",
  "status": "running",
  "book_id": 12,
  "chapter_id": 101,
  "progress": 42,
  "done_items": 18,
  "total_items": 42,
  "failed_items": 0,
  "current_step": "segment 18/42",
  "error_message": null,
  "created_at": "2026-11-12T09:00:00Z",
  "started_at": "2026-11-12T09:00:02Z",
  "finished_at": null
}
```

#### GET `/jobs/{id}`

查询：`include_items=true` 时附带 `items`（最多 500 条）。

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "job": { "id": 601, "job_type": "chapter_tts", "status": "partial", "progress": 100, "done_items": 40, "failed_items": 2, "total_items": 42, "result": { "chapter_audio_stale": true } },
    "items": [
      { "item_type": "segment", "item_id": 9003, "status": "failed", "attempts": 3, "error_message": "TTS timeout" }
    ]
  },
  "request_id": "..."
}
```

#### POST `/jobs/{id}/cancel`

响应 `200`：

```json
{ "code": 0, "message": "ok", "data": { "id": 601, "status": "running", "cancel_requested": true }, "request_id": "..." }
```

错误：`20003`（任务已结束）。

#### POST `/jobs/{id}/retry`

请求：

```json
{ "retry_failed_only": true }
```

响应 `202`：`{ "job_id": 602, "source_job_id": 601 }`。

错误：`20003`（仅 `failed/partial` 可重试）。

#### GET `/jobs/{id}/events`（SSE）

响应头：`Content-Type: text/event-stream`、`Cache-Control: no-cache`、`Connection: keep-alive`。

事件格式：

```text
event: progress
id: 601-42
data: {"job_id":601,"status":"running","progress":42,"done_items":18,"total_items":42,"failed_items":0,"current_step":"segment 18/42","updated_at":"2026-11-12T09:05:00Z"}

event: done
data: {"job_id":601,"status":"partial","progress":100,"done_items":40,"failed_items":2,"total_items":42,"result":{"failed_segment_ids":[9003,9017]}}

: ping
```

规则：

1. 连接建立后立即发送一次当前快照。
2. 任务进入终态（`succeeded/partial/failed/canceled`）后发送 `done` 并关闭。
3. 每 15s 发送注释心跳。
4. 客户端断开后可直接重连，服务端重新发送当前快照。

### 3.8 导出

#### POST `/books/{book_id}/exports`

请求：

```json
{ "include_cover": true, "chapter_ids": null }
```

`chapter_ids=null` 表示全部已拼接章节；指定时仅导出给定章节。

响应 `202`：

```json
{ "code": 0, "message": "accepted", "data": { "export_id": 31, "job_id": 901 }, "request_id": "..." }
```

错误：`20003` 无已拼接章节；`40004` 已有进行中的导出任务。

#### GET `/exports/{id}`

响应 `200`：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 31,
    "book_id": 12,
    "export_type": "book_zip",
    "status": "succeeded",
    "total_chapters": 30,
    "included_chapters": 30,
    "file_id": 501,
    "download_url": "/api/v1/exports/31/download",
    "error_message": null,
    "created_at": "2026-11-12T10:00:00Z",
    "finished_at": "2026-11-12T10:01:20Z"
  },
  "request_id": "..."
}
```

#### GET `/exports/{id}/download`

响应 `200`：`Content-Type: application/zip`，`Content-Disposition: attachment; filename="山海拾遗.zip"`。

错误：`20003`（未完成）；`20001`（文件不存在）。

### 3.9 文件

#### GET `/files/{id}`

鉴权下载。查询参数 `download=1` 时返回 attachment，否则 `inline`。

响应 `200`：按 `files.mime` 返回二进制。

错误：`404`。

### 3.10 健康检查

#### GET `/healthz`

```json
{ "status": "ok", "version": "0.1.0", "uptime_secs": 3600 }
```

#### GET `/readyz`

```json
{
  "status": "ok",
  "checks": { "mysql": "ok", "rustfs": "ok", "ffmpeg": "ok" }
}
```

依赖异常时返回 `503`，`checks` 中对应项为 `error: ...`。

## 4. 关键校验规则汇总

| 对象 | 规则 |
| --- | --- |
| 书名 | 1–255 字符；同库未删除范围不重名 |
| 章节标题 | 1–255 字符 |
| 段落文本 | 1–2000 字符（切段默认 ≤300，允许人工编辑更长但合成前校验） |
| 角色名 | 1–64 字符；同书唯一；别名 ≤20 个 |
| `rate_percent` | -50..100 |
| `pitch_hz` | -50..50 |
| `volume_percent` | -50..50 |
| `max_chars` | 50..1000 |
| 导入文件 | `.txt`，≤50MB |
| 封面 | `.jpg/.jpeg/.png/.webp`，≤5MB |
| 导出章节数 | 1..2000 |
| 批量段落更新 | 1..200 条 |

## 5. OpenAPI 与文档同步

1. 所有 Handler 使用 `utoipa` 标注请求/响应/错误码。
2. CI 中 `cargo test` 包含 OpenAPI 生成测试，确保 schema 可序列化。
3. 接口变更必须先改本文档，再改代码；PR 评审以文档为基准。
4. 生产环境默认关闭 `/docs` 与 `/api-docs`。

## 6. 评审关注点

1. 统一响应包络 `code/message/data/request_id` 是否符合预期。
2. 异步任务统一返回 `202 + job_id + SSE` 的交互方式是否接受。
3. 重复任务返回 `409/40004`（而非复用旧任务）是否符合预期。
4. 单段落合成也走异步任务，是否接受（便于统一进度与重试）。
