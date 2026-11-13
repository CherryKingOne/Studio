# 当前用户信息

获取当前登录用户的基本信息，常用于前端校验 Token 有效性、展示用户资料。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/auth/me` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

无（用户身份从 Token 中解析）。

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 用户 ID |
| username | string | 用户名 |
| email | string / null | 邮箱（历史遗留账号可能为 `null`） |
| email_verified | boolean | 邮箱是否已验证 |
| display_name | string / null | 显示名称 |
| role | string | 角色，如 `admin` / `user` |
| last_login_at | string / null | 上次登录时间（ISO 8601 UTC），从未登录为 `null` |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 1,
    "username": "admin",
    "email": "admin@localhost",
    "email_verified": true,
    "display_name": "管理员",
    "role": "admin",
    "last_login_at": "2026-11-13T08:30:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | Token 对应的用户已被删除 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/auth/me' \
  -H 'Authorization: Bearer <access_token>'
```