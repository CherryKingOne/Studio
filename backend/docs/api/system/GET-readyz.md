# 就绪探针（Readiness）

检查服务全部外部依赖是否可用：MySQL 数据库、RustFS 对象存储、FFmpeg。全部正常返回 `200`，任一异常返回 `503`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/readyz` |
| 认证 | 无需认证 |
| Content-Type | 无请求体 |

## 请求参数

无。

## 响应参数

HTTP `200`（全部依赖正常）或 `503`（存在异常依赖），直接返回 JSON（**不使用** `code/message/data` 包装结构）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| status | string | 整体状态：`ok`（全部正常）/ `degraded`（存在异常） |
| checks | object | 各依赖检查结果，见下表 |

`checks` 对象字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| mysql | string | MySQL 连接检查：`ok` 或 `error: <错误详情>` |
| rustfs | string | RustFS 对象存储检查：`ok` 或 `error: <错误详情>` |
| ffmpeg | string | FFmpeg 可执行检查（执行 `ffmpeg -version`）：`ok` 或 `error: ffmpeg unavailable` |

## 响应示例

全部正常（HTTP 200）：

```json
{
  "status": "ok",
  "checks": {
    "mysql": "ok",
    "rustfs": "ok",
    "ffmpeg": "ok"
  }
}
```

依赖异常（HTTP 503）：

```json
{
  "status": "degraded",
  "checks": {
    "mysql": "ok",
    "rustfs": "error: connection refused",
    "ffmpeg": "error: ffmpeg unavailable"
  }
}
```

## curl 示例

```bash
curl -i -X GET 'http://localhost:60181/readyz'
```

> `-i` 用于查看 HTTP 状态码，便于区分 `200` 与 `503`。