# 音色试听

实时合成一段试听音频并返回 MP3 二进制流，用于挑选音色、调试语速 / 音调 / 音量。按客户端 IP 限流。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/voices/preview` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |
| 限流 | 按客户端 IP 限流（取 `X-Forwarded-For` 第一个值），触发返回 `429`（10004） |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| voice_id | integer | 是 | - | 音色 ID |
| text | string | 否 | 你好，这是一段音色试听。 | 试听文本，去除首尾空白后不超过 100 个字符；空白按默认文本处理 |
| rate_percent | integer | 否 | 0 | 语速偏移（%） |
| pitch_hz | integer | 否 | 0 | 音调偏移（Hz） |
| volume_percent | integer | 否 | 0 | 音量偏移（%） |

## 响应参数

HTTP `200`，**二进制音频流**（不使用 JSON 包装结构）：

| 响应头 | 值 | 说明 |
| --- | --- | --- |
| Content-Type | `audio/mpeg` | MP3 音频 |
| Content-Length | integer | 音频字节数 |

响应体：实时合成的 MP3 二进制内容（不落库、不缓存）。

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
| 400 | 20002 | 试听文本超过 100 个字符 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 音色不存在 |
| 429 | 10004 | 同一 IP 试听请求过于频繁 |
| 429 | 40002 | TTS 引擎侧限流 |
| 502 | 40001 | TTS 引擎合成失败 |

## curl 示例

默认文本试听：

```bash
curl -X POST 'http://localhost:60181/api/v1/voices/preview' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"voice_id": 5}' \
  -o preview.mp3
```

自定义文本与语速：

```bash
curl -X POST 'http://localhost:60181/api/v1/voices/preview' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "voice_id": 5,
    "text": "清晨的雾气还未散去。",
    "rate_percent": 10,
    "pitch_hz": 0,
    "volume_percent": 0
  }' \
  -o preview.mp3
```