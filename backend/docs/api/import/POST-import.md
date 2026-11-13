# 导入 TXT 书籍

上传 TXT 文本文件，自动创建书籍并投递异步解析任务（`import_parse`）：识别编码、切分章节、切分段落。接口立即返回 `202` 与任务 ID，解析进度通过任务中心接口查询。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books/import` |
| 认证 | Bearer Token |
| Content-Type | `multipart/form-data` |
| 大小限制 | 路由层 60 MB；业务层 `MAX_IMPORT_BYTES`（默认 50 MB），以较小者为准 |

## 请求参数

### Form 参数（multipart/form-data）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| file | file | 是 | - | TXT 文本文件，扩展名必须为 `.txt`（不区分大小写），内容不能为空；支持 UTF-8 / GBK 等常见编码自动识别 |
| title | string | 否 | 文件名（去扩展名） | 书名，去除首尾空白；为空时回退为文件名主干，仍为空则用 `未命名书籍`；不能超过 255 个字符 |
| author | string | 否 | null | 作者，空白字符串按 `null` 处理 |

## 响应参数

HTTP `202 Accepted`，统一包装结构（`message` 为 `accepted`），`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| book_id | integer | 新创建的书籍 ID |
| job_id | integer | 异步解析任务 ID（`job_type` 为 `import_parse`），可用 `GET /api/v1/jobs/{job_id}` 或 SSE 接口跟踪进度 |

任务完成后，任务的 `result` 字段包含：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| book_id | integer | 书籍 ID |
| chapters | integer | 解析出的章节数 |
| segments | integer | 切分出的段落总数 |
| encoding | string | 识别到的源文件编码，如 `Utf8`、`Gbk` |

## 响应示例

```json
{
  "code": 0,
  "message": "accepted",
  "data": {
    "book_id": 1,
    "job_id": 101
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 缺少 file 字段、文件内容为空、书名超过 255 字符 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 413 | 30002 | 文件超过大小限制 |
| 415 | 30001 | 文件扩展名不是 `.txt` |
| 502 | 30003 | 对象存储写入失败 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/books/import' \
  -H 'Authorization: Bearer <access_token>' \
  -F 'file=@/path/to/novel.txt;type=text/plain' \
  -F 'title=示例小说' \
  -F 'author=佚名'
```

跟踪解析进度：

```bash
curl -X GET 'http://localhost:60181/api/v1/jobs/101' \
  -H 'Authorization: Bearer <access_token>'
```

> 导入期间书籍状态依次为 `importing` → `parsing` → `segmenting` → `ready`；解析失败时状态为 `failed`，原因见书籍详情的 `error_message`。