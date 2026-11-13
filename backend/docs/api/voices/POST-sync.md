# 同步供应商音色

调用 TTS 供应商的音色列表接口，将内置音色批量同步（Upsert）到本地音色库。同步为同步操作，直接返回结果。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/voices/sync` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| provider | string | 否 | 系统默认供应商（`TTS_DEFAULT_PROVIDER`，默认 `openai`） | 要同步的供应商：`openai` / `minimax` / `stepfun` |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| provider | string | 实际同步的供应商 |
| synced | integer | 本次同步的音色数量 |
| message | string | 结果描述：`已同步 N 个音色`；供应商无内置列表时为「该供应商不提供内置音色列表，请通过创建音色接口手动登记」 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "provider": "openai",
    "synced": 9,
    "message": "已同步 9 个音色"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

供应商无内置音色列表：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "provider": "minimax",
    "synced": 0,
    "message": "该供应商不提供内置音色列表，请通过创建音色接口手动登记"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | provider 不是受支持的 TTS 引擎（`details` 指明 `provider` 字段） |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 502 | 40001 | 供应商不可用或音色列表请求失败（如 API Key 未配置） |

## curl 示例

同步默认供应商：

```bash
curl -X POST 'http://localhost:60181/api/v1/voices/sync' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{}'
```

指定供应商：

```bash
curl -X POST 'http://localhost:60181/api/v1/voices/sync' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"provider": "openai"}'
```