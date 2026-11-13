# 服务信息

获取服务名称、版本、运行时长等基本信息，可用于前端展示或部署验证。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/` |
| 认证 | 无需认证 |
| Content-Type | 无请求体 |

## 请求参数

无。

## 响应参数

HTTP `200`，直接返回 JSON（**不使用** `code/message/data` 包装结构）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| service | string | 服务名称，固定为 `novelcast-server` |
| version | string | 服务版本号（Cargo 包版本），如 `0.1.0` |
| status | string | 服务状态，固定为 `ok` |
| api_base | string | 业务 API 前缀，固定为 `/api/v1` |
| health | string | 存活探针路径，固定为 `/healthz` |
| ready | string | 就绪探针路径，固定为 `/readyz` |
| uptime_secs | integer | 服务已运行秒数 |

## 响应示例

```json
{
  "service": "novelcast-server",
  "version": "0.1.0",
  "status": "ok",
  "api_base": "/api/v1",
  "health": "/healthz",
  "ready": "/readyz",
  "uptime_secs": 3600
}
```

## curl 示例

```bash
curl -X GET 'http://localhost:60181/'
```