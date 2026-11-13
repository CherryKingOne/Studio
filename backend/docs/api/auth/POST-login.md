# 登录

使用邮箱 + 密码登录，成功后签发双 Token：短期 `access_token`（JWT）与长期 `refresh_token`（不透明随机串）。业务接口携带 access_token；access_token 过期后用 refresh_token 调用 [刷新令牌](POST-refresh.md) 静默续期。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/login` |
| 认证 | 无需认证 |
| Content-Type | `application/json` |
| 限流 | 按 IP 每分钟 10 次 |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| email | string | 是 | - | 注册邮箱，自动转小写。初始管理员邮箱由 `ADMIN_EMAIL` 配置（默认 `admin@localhost`） |
| password | string | 是 | - | 密码，不能为空。初始管理员密码由 `ADMIN_PASSWORD` 配置 |

### 相关请求头

| 请求头 | 必填 | 说明 |
| --- | --- | --- |
| X-Forwarded-For | 否 | 反向代理传递的客户端 IP，取第一个值用于登录限流；缺失时按 `unknown` 计 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（SessionResponse）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| access_token | string | JWT 短期令牌，用于业务接口认证 |
| refresh_token | string | 不透明长期令牌（64 位十六进制），仅用于刷新 / 登出接口 |
| token_type | string | 固定为 `Bearer` |
| expires_in | integer | access_token 有效期（秒），默认 7200（2 小时，`JWT_TTL_HOURS`） |
| refresh_expires_in | integer | refresh_token 有效期（秒），默认 2592000（30 天，`JWT_REFRESH_TTL_DAYS`） |
| user | object | 用户信息，见下表 |

`user` 对象字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 用户 ID |
| username | string | 用户名 |
| email | string / null | 邮箱（历史遗留账号可能为 `null`，此类账号无法邮箱登录） |
| email_verified | boolean | 邮箱是否已验证 |
| display_name | string / null | 显示名称，如 `管理员` |
| role | string | 角色，如 `admin` / `user` |
| last_login_at | string / null | 上次登录时间（ISO 8601 UTC），首次登录为 `null` |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "9f2c7a1e4b8d3f6a0c5e9d2b7a4f1c8e3d6b9a0f5e2c7d4b1a8f3e6c9d0b5a2f",
    "token_type": "Bearer",
    "expires_in": 7200,
    "refresh_expires_in": 2592000,
    "user": {
      "id": 1,
      "username": "admin",
      "email": "admin@localhost",
      "email_verified": true,
      "display_name": "管理员",
      "role": "admin",
      "last_login_at": "2026-11-13T08:30:00Z"
    }
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 邮箱格式不正确或密码为空 |
| 401 | 10003 | 邮箱不存在或密码错误 |
| 429 | 10004 | 同一 IP 登录请求过于频繁 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/login' \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "admin@localhost",
    "password": "admin123456"
  }'
```

提取双 Token 供后续使用（需安装 jq）：

```bash
RESP=$(curl -s -X POST 'http://localhost:60181/api/v1/auth/login' \
  -H 'Content-Type: application/json' \
  -d '{"email": "admin@localhost", "password": "admin123456"}')
TOKEN=$(echo "$RESP" | jq -r '.data.access_token')
REFRESH=$(echo "$RESP" | jq -r '.data.refresh_token')
```

> 每次登录都会签发新的 refresh_token，旧 refresh_token 不受影响（多设备并行会话）。修改密码或重置密码会吊销该用户全部 refresh_token。