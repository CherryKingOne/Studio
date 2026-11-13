# 修改密码

修改当前登录用户的密码。需校验旧密码，新密码至少 8 个字符。成功后返回 `204`，无响应体。

**副作用**：修改成功后**吊销该用户全部 refresh_token**（所有设备会话失效，需重新登录）；已签发的 access_token 在有效期内仍可用。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/password` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| old_password | string | 是 | - | 当前密码，用于身份校验；错误时返回 `10003` |
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
| 400 | 20002 | 新密码少于 8 个字符（`details` 指明 `new_password`） |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 401 | 10003 | 旧密码错误 |
| 404 | 20001 | Token 对应的用户已被删除 |

## curl 示例

```bash
curl -i -X POST 'http://localhost:60181/api/v1/auth/password' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "old_password": "old-password",
    "new_password": "new-password-123"
  }'
```

> 修改密码会吊销该用户全部 refresh_token，其他设备需重新登录；已签发的 access_token 在有效期内（默认 2 小时）仍可使用。