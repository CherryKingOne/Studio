# 登出

吊销指定的 refresh_token，结束该会话的续期能力。成功后返回 `204`，无响应体。接口幂等：重复登出或传入无效 token 同样返回 `204`。

**注意**：登出只吊销 refresh_token；已签发的 access_token 在有效期内（默认 2 小时）仍然可用，前端应同时清除本地存储的两个 Token。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/logout` |
| 认证 | 无需 Bearer Token（凭 refresh_token 本身鉴权） |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| refresh_token | string | 是 | - | 要吊销的 refresh_token |

## 响应参数

HTTP `204 No Content`，**无响应体**。

## 响应示例

```
HTTP/1.1 204 No Content
x-request-id: 01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | — | 请求体不是合法 JSON 或缺少 refresh_token 字段（框架默认拒绝响应，非统一包装结构） |

## curl 示例

```bash
curl -i -X POST 'http://localhost:60181/api/v1/auth/logout' \
  -H 'Content-Type: application/json' \
  -d '{"refresh_token": "9f2c7a1e4b8d3f6a0c5e9d2b7a4f1c8e3d6b9a0f5e2c7d4b1a8f3e6c9d0b5a2f"}'
```

> 如需吊销某用户的全部会话（所有设备），请使用修改密码或重置密码接口。