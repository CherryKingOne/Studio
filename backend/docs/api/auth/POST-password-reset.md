# 找回密码 — 重置密码

凭邮箱验证码设置新密码。重置成功后**吊销该用户全部 refresh_token**（所有设备会话失效，需重新登录）。成功后返回 `204`，无响应体。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/password/reset` |
| 认证 | 无需认证（凭邮箱验证码鉴权） |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| email | string | 是 | - | 已注册邮箱，自动转小写 |
| code | string | 是 | - | [发送重置验证码](POST-password-forgot.md) 获取的 6 位验证码，5 分钟内有效，最多尝试 5 次 |
| new_password | string | 是 | - | 新密码，至少 8 个字符（按 Unicode 字符计数） |

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
| 400 | 20002 | 邮箱格式不正确；新密码少于 8 个字符；验证码为空 / 错误 / 过期 / 尝试次数过多 |
| 404 | 20001 | 该邮箱未注册 |

## curl 示例

```bash
curl -i -X POST 'http://localhost:60181/api/v1/auth/password/reset' \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "user@example.com",
    "code": "730215",
    "new_password": "new-password-456"
  }'
```

重置后重新登录：

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/login' \
  -H 'Content-Type: application/json' \
  -d '{"email": "user@example.com", "password": "new-password-456"}'
```

> 验证码一次性有效：重置成功后立即作废。已签发的 access_token 在有效期内仍可用（默认 2 小时），但无法再续期。