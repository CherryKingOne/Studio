# 音色示例音频

获取音色的示例 MP3 音频。首次请求时实时合成默认试听文本（「你好，这是一段音色试听。」）并缓存到对象存储，后续请求直接返回缓存文件。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/voices/{id}/sample` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 音色 ID |

## 响应参数

HTTP `200`，**二进制音频流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | `audio/mpeg` | MP3 音频 |
| Content-Length | integer | 音频字节数 |

响应体：示例 MP3 二进制内容。

## 响应示例

```
HTTP/1.1 200 OK
content-type: audio/mpeg
content-length: 48000

<二进制 MP3 数据>
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 音色不存在 |
| 429 | 40002 | TTS 引擎侧限流（首次合成时） |
| 502 | 40001 | TTS 引擎合成失败（首次合成时） |
| 502 | 30003 | 对象存储读写失败 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/voices/5/sample' \
  -H 'Authorization: Bearer <access_token>' \
  -o voice-5-sample.mp3
```

> 首次请求会触发一次真实 TTS 合成，耗时较长；之后请求命中缓存，响应很快。