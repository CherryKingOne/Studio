# NovelCast 后端 API 文档

`novelcast-server`（有声小说制作平台后端）全部 HTTP 接口文档。按路由资源划分子文件夹，每个接口一个 Markdown 文档，包含请求参数表、响应参数表、响应示例与 curl 请求示例。

## 通用约定

### 服务地址

- 默认基地址：`http://localhost:60181`
- 端口由环境变量 `APP_PORT` 配置（默认 `60181`），监听地址由 `APP_HOST` 配置（默认 `0.0.0.0`）。
- 业务接口统一前缀 `/api/v1`；系统探针接口（`/`、`/healthz`、`/readyz`）无前缀。
- 下文 curl 示例中的 `<access_token>` 请替换为登录接口返回的真实 Token。

### 认证方式（双 Token）

除 **注册 / 登录 / 验证码 / 刷新 / 登出 / 找回密码**、**系统探针**（`/`、`/healthz`、`/readyz`）、**文件下载**（`GET /api/v1/files/{id}`）外，所有接口都需要在请求头携带短期 JWT：

```
Authorization: Bearer <access_token>
```

会话采用 **access_token + refresh_token** 双 Token 机制：

| Token | 形态 | 默认有效期 | 用途 | 配置项 |
| --- | --- | --- | --- | --- |
| access_token | JWT | 2 小时 | 业务接口认证 | `JWT_TTL_HOURS` |
| refresh_token | 不透明随机串（64 位十六进制，服务端哈希存储） | 30 天 | 刷新续期 / 登出吊销 | `JWT_REFRESH_TTL_DAYS` |

- 注册（邮箱 + 密码 + 邮箱验证码）或登录（邮箱 + 密码）成功后同时签发两个 Token。
- access_token 过期后调用 `POST /api/v1/auth/refresh` 换取新 access_token，refresh_token 同时**轮换**（旧值作废）；已吊销 refresh_token 被重用时，该用户全部会话被吊销。
- 修改密码 / 重置密码会吊销该用户全部 refresh_token；登出仅吊销传入的 refresh_token。
- 缺失或无效 access_token 返回 `401`，业务码 `10001`。

邮箱验证码通过 SMTP 发送（`SMTP_HOST` 等配置）；未配置 SMTP 时为开发模式，验证码输出到服务端日志，且 `APP_ENV=development` 时发送验证码接口额外返回 `dev_code` 字段。

### 统一响应结构（业务接口）

成功响应（HTTP `200` / `201` / `202`）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| code | integer | 业务码，成功固定为 `0` |
| message | string | 结果描述：`ok`（200）、`created`（201）、`accepted`（202，异步任务已受理） |
| data | object / array | 业务数据，各接口不同 |
| request_id | string | 本次请求的追踪 ID（UUID v7） |

失败响应（HTTP `4xx` / `5xx`）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| code | integer | 业务错误码，见下表 |
| message | string | 错误描述（中文） |
| details | array / object / null | 附加信息：参数校验错误为 `[{field, message}]` 数组；任务冲突为 `{existing_job_id}`；其余为 `null` |
| request_id | string | 本次请求的追踪 ID |

> 注意：系统探针接口（`/`、`/healthz`、`/readyz`）与二进制下载接口（音频、封面、导出 zip）**不使用**上述包装结构，直接返回原始 JSON 或文件流。`204 No Content` 响应无响应体。

### 全局错误码

| HTTP 状态码 | code | 含义 | 说明 |
| --- | --- | --- | --- |
| 401 | 10001 | 未认证或 Token 无效 | 缺少 / 错误的 `Authorization` 头 |
| 403 | 10002 | 无权限 | 角色权限不足 |
| 401 | 10003 | 用户名或密码错误 | 登录失败、旧密码校验失败 |
| 429 | 10004 | 请求过于频繁 | 登录 / 音色试听按 IP 限流 |
| 404 | 20001 | 资源不存在 | 书籍 / 章节 / 段落 / 任务等不存在，`message` 指明资源 |
| 400 | 20002 | 参数校验失败 | `details` 为字段级错误数组 |
| 409 | 20003 | 状态冲突 | 当前资源状态不允许该操作 |
| 409 | 20004 | 资源重复 | 同名书籍、同书角色名已存在等 |
| 409 | 20005 | 资源被占用 | 资源仍被引用，无法删除 |
| 415 | 30001 | 文件类型不支持 | 导入仅支持 `.txt`；封面仅支持 jpg/png/webp |
| 413 | 30002 | 文件过大 | 超过对应上传限制 |
| 502 | 30003 | 对象存储错误 | RustFS 读写失败 |
| 502 | 40001 | TTS 引擎错误 | 语音合成供应商返回错误 |
| 429 | 40002 | TTS 引擎限流 | 供应商侧限流 |
| 500 | 40003 | 音频处理失败 | FFmpeg 处理错误 |
| 409 | 40004 | 已有进行中的同类任务 | `details.existing_job_id` 为进行中任务 ID |
| 500 | 50001 | 内部错误 | 服务端未预期错误 |

### 分页约定

列表接口（书籍、章节、音色、任务）使用统一分页参数：

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| page | integer | 否 | 1 | 页码，从 1 开始，小于 1 按 1 处理 |
| page_size | integer | 否 | 20 | 每页条数，限制 1~100（音色列表默认 50，上限 200） |

分页响应的 `data` 结构：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 当前页数据列表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 满足条件的总记录数 |

> 段落列表与角色列表不分页，返回 `{items, total}` 全量数据。

### 时间格式

所有时间字段均为 ISO 8601 UTC 字符串：`YYYY-MM-DDTHH:MM:SSZ`，例如 `2026-11-13T08:30:00Z`。可空时间字段为 `null`。

### 请求追踪

- 每个响应都带 `x-request-id` 响应头，JSON 响应体中同时包含 `request_id` 字段。
- 可通过请求头 `X-Request-Id` 自行指定（仅限字母、数字、`-`、`_`，长度 ≤ 128），否则服务端自动生成 UUID v7。

### 请求体大小限制

| 接口 | 限制 | 说明 |
| --- | --- | --- |
| 全局默认 | 2 MB | 普通 JSON 请求体 |
| `POST /api/v1/books/import` | 60 MB（路由层）| 业务层另校验 `MAX_IMPORT_BYTES`（默认 50 MB） |
| `POST /api/v1/books/{id}/cover` | 10 MB（路由层）| 业务层另校验 `MAX_IMAGE_BYTES`（默认 5 MB） |

## 接口索引

### system — 系统探针（无需认证）

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-index.md](system/GET-index.md) | GET | `/` | 服务信息 |
| [GET-healthz.md](system/GET-healthz.md) | GET | `/healthz` | 存活探针 |
| [GET-readyz.md](system/GET-readyz.md) | GET | `/readyz` | 就绪探针（依赖检查） |

### auth — 账户认证（邮箱 + 双 Token）

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [POST-register-code.md](auth/POST-register-code.md) | POST | `/api/v1/auth/register/code` | 发送注册邮箱验证码 |
| [POST-register.md](auth/POST-register.md) | POST | `/api/v1/auth/register` | 邮箱 + 密码 + 验证码注册 |
| [POST-login.md](auth/POST-login.md) | POST | `/api/v1/auth/login` | 邮箱 + 密码登录（签发双 Token） |
| [POST-refresh.md](auth/POST-refresh.md) | POST | `/api/v1/auth/refresh` | 刷新 access_token（轮换 refresh_token） |
| [POST-logout.md](auth/POST-logout.md) | POST | `/api/v1/auth/logout` | 登出（吊销 refresh_token） |
| [GET-me.md](auth/GET-me.md) | GET | `/api/v1/auth/me` | 当前用户信息 |
| [POST-password.md](auth/POST-password.md) | POST | `/api/v1/auth/password` | 修改密码（吊销全部会话） |
| [POST-password-forgot.md](auth/POST-password-forgot.md) | POST | `/api/v1/auth/password/forgot` | 找回密码：发送重置验证码 |
| [POST-password-reset.md](auth/POST-password-reset.md) | POST | `/api/v1/auth/password/reset` | 找回密码：验证码重置密码 |

### books — 书籍管理

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](books/GET-list.md) | GET | `/api/v1/books` | 书籍列表（分页） |
| [POST-create.md](books/POST-create.md) | POST | `/api/v1/books` | 创建书籍 |
| [GET-detail.md](books/GET-detail.md) | GET | `/api/v1/books/{id}` | 书籍详情 |
| [PATCH-update.md](books/PATCH-update.md) | PATCH | `/api/v1/books/{id}` | 更新书籍 |
| [DELETE-delete.md](books/DELETE-delete.md) | DELETE | `/api/v1/books/{id}` | 删除书籍（软删除） |
| [POST-cover.md](books/POST-cover.md) | POST | `/api/v1/books/{id}/cover` | 上传封面 |
| [GET-stats.md](books/GET-stats.md) | GET | `/api/v1/books/{id}/stats` | 书籍制作统计 |

### import — 文本导入

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [POST-import.md](import/POST-import.md) | POST | `/api/v1/books/import` | 导入 TXT 并异步解析切章 |

### chapters — 章节管理

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](chapters/GET-list.md) | GET | `/api/v1/books/{book_id}/chapters` | 章节列表（分页） |
| [GET-detail.md](chapters/GET-detail.md) | GET | `/api/v1/chapters/{id}` | 章节详情 |
| [PATCH-update.md](chapters/PATCH-update.md) | PATCH | `/api/v1/chapters/{id}` | 更新章节标题 / 正文 |
| [POST-resegment.md](chapters/POST-resegment.md) | POST | `/api/v1/chapters/{id}/resegment` | 重新切段 |

### segments — 段落管理

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](segments/GET-list.md) | GET | `/api/v1/chapters/{chapter_id}/segments` | 段落列表（含解析音色） |
| [PATCH-update.md](segments/PATCH-update.md) | PATCH | `/api/v1/segments/{id}` | 更新单个段落 |
| [PATCH-batch.md](segments/PATCH-batch.md) | PATCH | `/api/v1/segments/batch` | 批量更新段落 |

### tts — 语音合成

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [POST-chapter.md](tts/POST-chapter.md) | POST | `/api/v1/chapters/{id}/tts` | 章节批量合成（异步） |
| [POST-segment.md](tts/POST-segment.md) | POST | `/api/v1/segments/{id}/tts` | 单段落合成（异步） |

### audio — 音频拼接与试听

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [POST-merge.md](audio/POST-merge.md) | POST | `/api/v1/chapters/{id}/merge` | 章节音频拼接（异步） |
| [GET-chapter.md](audio/GET-chapter.md) | GET | `/api/v1/chapters/{id}/audio` | 下载章节音频 |
| [DELETE-chapter.md](audio/DELETE-chapter.md) | DELETE | `/api/v1/chapters/{id}/audio` | 删除章节音频 |
| [GET-segment.md](audio/GET-segment.md) | GET | `/api/v1/segments/{id}/audio` | 下载段落音频 |

### exports — 整书导出

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [POST-create.md](exports/POST-create.md) | POST | `/api/v1/books/{id}/exports` | 创建导出任务（异步） |
| [GET-detail.md](exports/GET-detail.md) | GET | `/api/v1/exports/{id}` | 导出记录详情 |
| [GET-download.md](exports/GET-download.md) | GET | `/api/v1/exports/{id}/download` | 下载导出 zip |

### characters — 角色管理

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](characters/GET-list.md) | GET | `/api/v1/books/{book_id}/characters` | 角色列表 |
| [POST-create.md](characters/POST-create.md) | POST | `/api/v1/books/{book_id}/characters` | 创建角色 |
| [PATCH-update.md](characters/PATCH-update.md) | PATCH | `/api/v1/characters/{id}` | 更新角色 |
| [DELETE-delete.md](characters/DELETE-delete.md) | DELETE | `/api/v1/characters/{id}` | 删除角色 |
| [POST-detect.md](characters/POST-detect.md) | POST | `/api/v1/books/{book_id}/characters/detect` | 角色自动识别（异步） |

### voices — 音色管理

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](voices/GET-list.md) | GET | `/api/v1/voices` | 音色列表（分页） |
| [POST-create.md](voices/POST-create.md) | POST | `/api/v1/voices` | 手动登记音色 |
| [POST-sync.md](voices/POST-sync.md) | POST | `/api/v1/voices/sync` | 同步供应商音色 |
| [POST-preview.md](voices/POST-preview.md) | POST | `/api/v1/voices/preview` | 音色试听（返回音频） |
| [GET-sample.md](voices/GET-sample.md) | GET | `/api/v1/voices/{id}/sample` | 音色示例音频 |

### jobs — 任务中心

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-list.md](jobs/GET-list.md) | GET | `/api/v1/jobs` | 任务列表（分页） |
| [GET-detail.md](jobs/GET-detail.md) | GET | `/api/v1/jobs/{id}` | 任务详情 |
| [POST-cancel.md](jobs/POST-cancel.md) | POST | `/api/v1/jobs/{id}/cancel` | 取消任务 |
| [POST-retry.md](jobs/POST-retry.md) | POST | `/api/v1/jobs/{id}/retry` | 重试任务 |
| [GET-events.md](jobs/GET-events.md) | GET | `/api/v1/jobs/{id}/events` | 任务进度 SSE 流 |

### files — 文件服务

| 文档 | 方法 | 路径 | 说明 |
| --- | --- | --- | --- |
| [GET-download.md](files/GET-download.md) | GET | `/api/v1/files/{id}` | 文件下载 / 内联预览（无需认证） |

## 状态枚举速查

| 资源 | 字段 | 取值 |
| --- | --- | --- |
| 书籍 | status | `draft`（草稿）、`importing`（导入中）、`parsing`（解析中）、`segmenting`（切章中）、`ready`（就绪）、`failed`（失败）、`archived`（已归档 / 软删除） |
| 章节 | status | `pending`（待切段）、`segmented`（已切段）、`tts_partial`（部分合成）、`tts_done`（全部合成）、`merged`（已拼接） |
| 段落 | status | `pending`（待合成）、`synthesizing`（合成中）、`done`（已完成）、`failed`（失败） |
| 段落 | seg_type | `narration`（旁白）、`dialogue`（对白） |
| 任务 | status | `queued`（排队）、`running`（执行中）、`succeeded`（成功）、`partial`（部分成功）、`failed`（失败）、`canceled`（已取消） |
| 任务 | job_type | `import_parse`（导入解析）、`chapter_tts`（章节合成）、`chapter_merge`（章节拼接）、`character_detect`（角色识别）、`book_export`（整书导出） |
| 导出 | status | `queued`（排队）、`running`（执行中）、`succeeded`（成功）、`failed`（失败） |
| 角色 | gender | `male`、`female`、`unknown` |
| 角色 | age_group | `child`、`teen`、`adult`、`elder`、`unknown` |
| 角色 | source | `manual`（手动创建）、`rule`（规则识别） |
| 音色解析 | resolved_voice.source | `fallback`（段落已指定音色）、`character_default`（角色默认音色）、`book_narrator`（章节 / 书籍旁白音色）、`system_default`（系统默认音色）、`missing_voice`（指定的音色不存在或未启用） |