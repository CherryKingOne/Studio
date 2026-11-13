# 存活探针（Liveness）

Kubernetes / 负载均衡存活探针。只要进程在运行就返回 `200`，不检查外部依赖。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/healthz` |
| 认证 | 无需认证 |
| Content-Type | 无请求体 |

## 请求参数

无。

## 响应参数

HTTP `200`，直接返回 JSON（**不使用** `code/message/data` 包装结构）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| status | string | 服务状态，固定为 `ok` |
| version | string | 服务版本号，如 `0.1.0` |
| uptime_secs | integer | 服务已运行秒数 |

## 响应示例

```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_secs": 3600
}
```

## curl 示例

```bash
curl -X GET 'http://localhost:60181/healthz'
```