# 下载段落音频

下载单个段落合成后的 MP3 音频，以二进制流返回（内联方式，可直接用于 `<audio>` 试听）。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/segments/{id}/audio` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 段落 ID |

## 响应参数

HTTP `200`，**二进制音频流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | `audio/mpeg` | MP3 音频 |
| Content-Disposition | `inline; filename*=UTF-8''segment-{id}.mp3` | 内联展示，文件名如 `segment-100.mp3` |
| Content-Length | integer | 音频字节数 |

响应体：MP3 文件二进制内容。

## 响应示例

```
HTTP/1.1 200 OK
content-type: audio/mpeg
content-disposition: inline; filename*=UTF-8''segment-100.mp3
content-length: 51200

<二进制 MP3 数据>
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 段落不存在，或段落尚未合成音频（`message` 为「段落音频 不存在」） |
| 502 | 30003 | 对象存储读取失败 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/segments/100/audio' \
  -H 'Authorization: Bearer <access_token>' \
  -o segment-100.mp3
```