# 发送邮箱验证码（注册 / 找回密码）

统一的邮箱验证码发送接口，通过 `purpose` 字段区分场景：注册（`register`）或找回密码（`reset_password`）。验证码为 6 位数字，5 分钟内有效，同一邮箱同一场景 60 秒内只能发送一次、每小时最多 5 次。

**场景差异：**

| purpose | 邮箱要求 | 用途 |
| --- | --- | --- |
| register | 必须**未注册**（已注册返回 `20004`） | 配合 [注册](POST-register.md) 接口校验邮箱所有权 |
| reset_password | 必须**已注册**（未注册返回 `20001`） | 配合 [重置密码](POST-password-reset.md) 接口找回密码 |

**开发模式**：未配置 `SMTP_HOST` 时不真正发信，验证码输出到服务端日志；且 `APP_ENV=development` 时响应额外返回 `dev_code` 字段便于调试。生产环境不会返回该字段。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/email/code` |
| 认证 | 无需认证 |
| Content-Type | `application/json` |
| 限流 | 按 IP + 场景每分钟 10 次；按邮箱 60 秒重发间隔、每小时 5 次 |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| email | string | 是 | - | 目标邮箱，自动转小写并校验格式（≤255 字符） |
| purpose | string | 是 | - | 验证码用途：`register`（注册）/ `reset_password`（找回密码），其他值返回 `20002` |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| expires_in | integer | 验证码有效期（秒），固定 300 |
| resend_after | integer | 重发间隔（秒），固定 60 |
| dev_code | string | 仅开发模式（未配置 SMTP 且 `APP_ENV=development`）返回的验证码明文；生产环境无此字段 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "expires_in": 300,
    "resend_after": 60,
    "dev_code": "482913"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 邮箱格式不正确；purpose 不是 `register` / `reset_password` |
| 404 | 20001 | `purpose=reset_password` 时该邮箱未注册 |
| 409 | 20004 | `purpose=register` 时该邮箱已注册 |
| 409 | 20003 | 60 秒内重复发送（`message` 提示等待秒数） |
| 429 | 10004 | 同一 IP 请求过频，或同一邮箱每小时发送超过 5 次 |
| 500 | 50001 | SMTP 已配置但邮件发送失败（详见服务端日志） |

## curl 示例

注册场景：

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/email/code' \
  -H 'Content-Type: application/json' \
  -d '{"email": "user@example.com", "purpose": "register"}'
```

找回密码场景：

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/email/code' \
  -H 'Content-Type: application/json' \
  -d '{"email": "user@example.com", "purpose": "reset_password"}'
```

> 两个场景的验证码相互独立：同一邮箱可以分别持有注册验证码与重置验证码，互不影响；验证码与场景绑定，注册验证码不能用于重置密码，反之亦然。