# 下载导出 ZIP

下载导出完成的整书 ZIP 包，以二进制流返回（附件方式，浏览器触发下载）。仅当导出状态为 `succeeded` 时可下载。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/exports/{id}/download` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 导出记录 ID |

## 响应参数

HTTP `200`，**二进制 ZIP 流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | `application/zip` | ZIP 压缩包 |
| Content-Disposition | `attachment; filename*=UTF-8''<书名>.zip` | 附件下载；文件名为书籍标题（URL 编码）+ `.zip` |
| Content-Length | integer | ZIP 字节数 |

响应体：ZIP 文件二进制内容，包含 `manifest.json`、各章 MP3、可选封面。

## 响应示例

```
HTTP/1.1 200 OK
content-type: application/zip
content-disposition: attachment; filename*=UTF-8''%E7%A4%BA%E4%BE%8B%E5%B0%8F%E8%AF%B4.zip
content-length: 104857600

<二进制 ZIP 数据>
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 导出记录不存在，或导出文件缺失 |
| 409 | 20003 | 导出尚未完成（状态不是 `succeeded`） |
| 502 | 30003 | 对象存储读取失败 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/exports/7/download' \
  -H 'Authorization: Bearer <access_token>' \
  -o novel-export.zip
```

保留服务端文件名下载：

```bash
curl -X GET 'http://localhost:60181/api/v1/exports/7/download' \
  -H 'Authorization: Bearer <access_token>' \
  -OJ
```