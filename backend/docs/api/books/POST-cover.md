# 上传封面

为书籍上传封面图片，使用 `multipart/form-data` 格式。支持 JPG / PNG / WebP，上传成功后自动关联到书籍并返回访问地址。重复上传会覆盖旧封面关联。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books/{id}/cover` |
| 认证 | Bearer Token |
| Content-Type | `multipart/form-data` |
| 大小限制 | 路由层 10 MB；业务层 `MAX_IMAGE_BYTES`（默认 5 MB），以较小者为准 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

### Form 参数（multipart/form-data）

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| file | file | 是 | 封面图片文件。类型判定优先取文件部分的 Content-Type（`image/jpeg` / `image/png` / `image/webp`），否则按扩展名（jpg/jpeg/png/webp）判定；两者都无法识别时返回 `30001`。其他表单字段会被忽略 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| cover_file_id | integer | 新封面文件 ID（files 表记录） |
| cover_url | string | 封面访问地址，格式 `/api/v1/files/{cover_file_id}`，可直接用于 `<img>` 标签（该接口无需认证） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "cover_file_id": 12,
    "cover_url": "/api/v1/files/12"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 缺少 `file` 字段或 multipart 解析失败 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已软删除 |
| 413 | 30002 | 图片超过大小限制 |
| 415 | 30001 | 图片类型不支持（非 jpg/png/webp） |
| 502 | 30003 | 对象存储写入失败 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/cover' \
  -H 'Authorization: Bearer <access_token>' \
  -F 'file=@/path/to/cover.jpg;type=image/jpeg'
```

上传 PNG：

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/cover' \
  -H 'Authorization: Bearer <access_token>' \
  -F 'file=@/path/to/cover.png;type=image/png'
```