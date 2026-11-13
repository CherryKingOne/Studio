# 下载章节音频

下载章节拼接后的整章 MP3 音频，以二进制流返回（内联方式，可直接用于 `<audio>` 播放）。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/chapters/{id}/audio` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 章节 ID |

## 响应参数

HTTP `200`，**二进制音频流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | `audio/mpeg` | MP3 音频 |
| Content-Disposition | `inline; filename*=UTF-8''<文件名>` | 内联展示；文件名为 `{三位序号}-{章节标题}.mp3`（URL 编码），如 `001-第一章 初入江湖.mp3` |
| Content-Length | integer | 音频字节数 |

响应体：MP3 文件二进制内容。

## 响应示例

```
HTTP/1.1 200 OK
content-type: audio/mpeg
content-disposition: inline; filename*=UTF-8''001-%E7%AC%AC%E4%B8%80%E7%AB%A0.mp3
content-length: 6720000

<二进制 MP3 数据>
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 章节不存在，或章节尚未拼接音频（`message` 为「章节音频 不存在」） |
| 502 | 30003 | 对象存储读取失败 |

## curl 示例

保存到本地文件：

```bash
curl -X GET 'http://localhost:60181/api/v1/chapters/10/audio' \
  -H 'Authorization: Bearer <access_token>' \
  -o chapter-10.mp3
```

查看响应头：

```bash
curl -i -X GET 'http://localhost:60181/api/v1/chapters/10/audio' \
  -H 'Authorization: Bearer <access_token>' \
  -o /dev/null
```