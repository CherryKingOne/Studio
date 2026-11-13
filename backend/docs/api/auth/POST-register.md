# 注册

使用邮箱 + 密码 + 邮箱验证码注册新用户。注册成功后邮箱即视为已验证，并直接签发双 Token（access_token + refresh_token），无需再次登录。新用户角色为 `user`，用户名自动从邮箱前缀派生。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/register` |
| 认证 | 无需认证 |
| Content-Type | `application/json` |
| 限流 | 按 IP 每分钟 10 次 |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| email | string | 是 | - | 邮箱，自动转小写；必须已调用 [发送注册验证码](POST-register-code.md) 获取验证码 |
| password | string | 是 | - | 密码，至少 8 个字符（按 Unicode 字符计数） |
| code | string | 是 | - | 邮箱收到的 6 位数字验证码，5 分钟内有效，最多尝试 5 次 |

## 响应参数

HTTP `201 Created`，统一包装结构（`message` 为 `created`），`data` 字段（SessionResponse）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| access_token | string | JWT 短期令牌，用于业务接口认证（`Authorization: Bearer <access_token>`） |
| refresh_token | string | 不透明长期令牌（64 位十六进制），仅用于刷新接口，服务端哈希存储 |
| token_type | string | 固定为 `Bearer` |
| expires_in | integer | access_token 有效期（秒），`JWT_TTL_HOURS × 3600`，默认 7200（2 小时） |
| refresh_expires_in | integer | refresh_token 有效期（秒），`JWT_REFRESH_TTL_DAYS × 86400`，默认 2592000（30 天） |
| user | object | 用户信息，见下表 |

`user` 对象字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 用户 ID |
| username | string | 用户名（从邮箱前缀派生，重名时追加随机后缀） |
| email | string | 注册邮箱（小写） |
| email_verified | boolean | 邮箱是否已验证，注册成功固定为 `true` |
| display_name | string / null | 显示名称，新注册为 `null` |
| role | string | 角色，新注册为 `user` |
| last_login_at | string / null | 上次登录时间（ISO 8601 UTC），新注册为 `null` |

## 响应示例

```json
{
  "code": 0,
  "message": "created",
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "9f2c7a1e4b8d3f6a0c5e9d2b7a4f1c8e3d6b9a0f5e2c7d4b1a8f3e6c9d0b5a2f",
    "token_type": "Bearer",
    "expires_in": 7200,
    "refresh_expires_in": 2592000,
    "user": {
      "id": 2,
      "username": "user",
      "email": "user@example.com",
      "email_verified": true,
      "display_name": null,
      "role": "user",
      "last_login_at": null
    }
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 邮箱格式不正确；密码少于 8 个字符；验证码为空 / 错误 / 过期 / 尝试次数过多 |
| 409 | 20004 | 该邮箱已注册 |
| 429 | 10004 | 同一 IP 注册请求过于频繁 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/register' \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "user@example.com",
    "password": "my-password-123",
    "code": "482913"
  }'
```

> 验证码一次性有效：注册成功后立即作废。注册失败（如密码太短）不消耗验证码，但错误验证码会计入尝试次数。