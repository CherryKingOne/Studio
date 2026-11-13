# 发送注册验证码

向待注册邮箱发送 6 位数字验证码，用于注册时校验邮箱所有权。验证码 5 分钟内有效，同一邮箱 60 秒内只能发送一次、每小时最多 5 次。

**开发模式**：未配置 `SMTP_HOST` 时不真正发信，验证码输出到服务端日志；且 `APP_ENV=development` 时响应额外返回 `dev_code` 字段便于调试。生产环境不会返回该字段。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/register/code` |
| 认证 | 无需认证 |
| Content-Type | `application/json` |
| 限流 | 按 IP 每分钟 10 次；按邮箱 60 秒重发间隔、每小时 5 次 |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| email | string | 是 | - | 待注册邮箱，自动转小写并校验格式（≤255 字符）；已注册的邮箱返回 `20004` |

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
| 400 | 20002 | 邮箱格式不正确 |
| 409 | 20004 | 该邮箱已注册 |
| 409 | 20003 | 60 秒内重复发送（`message` 提示等待秒数） |
| 429 | 10004 | 同一 IP 请求过频，或同一邮箱每小时发送超过 5 次 |
| 500 | 50001 | SMTP 已配置但邮件发送失败（详见服务端日志） |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/register/code' \
  -H 'Content-Type: application/json' \
  -d '{"email": "user@example.com"}'
```