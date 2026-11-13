# 文件下载 / 预览

按文件 ID 下载或内联预览对象存储中的文件（封面、源 TXT、段落 / 章节音频、导出 ZIP 等）。**无需认证**，可直接用于 `<img>`、`<audio>` 标签。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/files/{id}` |
| 认证 | 无需认证 |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 文件 ID（files 表记录，如书籍详情中的 `cover_file_id`） |

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| download | integer | 否 | 0 | `1` 时以附件方式下载（`Content-Disposition: attachment`）；`0` 或缺省时内联预览（`inline`） |

## 响应参数

HTTP `200`，**二进制文件流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | 文件记录的 MIME | 如 `image/jpeg`、`audio/mpeg`、`text/plain`、`application/zip`；无法识别时为 `application/octet-stream` |
| Content-Disposition | `inline` 或 `attachment`; `filename*=UTF-8''<文件名>` | 由 `download` 参数决定；文件名为上传时的原始文件名（URL 编码） |
| Content-Length | integer | 文件字节数 |

响应体：文件二进制内容。

## 响应示例

```
HTTP/1.1 200 OK
content-type: image/jpeg
content-disposition: inline; filename*=UTF-8''cover.jpg
content-length: 204800

<二进制图片数据>
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 404 | 20001 | 文件记录不存在 |
| 502 | 30003 | 对象存储读取失败 |

## curl 示例

内联预览封面（浏览器直接打开亦可）：

```bash
curl -X GET 'http://localhost:60181/api/v1/files/12' -o cover.jpg
```

以附件方式下载：

```bash
curl -X GET 'http://localhost:60181/api/v1/files/12?download=1' -OJ
```

前端用法示例：

```html
<img src="http://localhost:60181/api/v1/files/12" alt="书籍封面" />
```