# 刷新令牌

用 refresh_token 换取新的 access_token，同时**轮换** refresh_token（旧 refresh_token 立即作废，返回新的）。用于 access_token 过期后的静默续期，无需重新登录。

**安全机制（重用检测）**：若一个已被吊销的 refresh_token 再次被使用，服务端会吊销该用户的**全部** refresh_token（判定为令牌泄露），所有会话需重新登录。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/auth/refresh` |
| 认证 | 无需 Bearer Token（凭 refresh_token 本身鉴权） |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| refresh_token | string | 是 | - | 登录 / 注册 / 上次刷新时返回的 refresh_token |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段（RefreshResponse）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| access_token | string | 新签发的 JWT 短期令牌 |
| refresh_token | string | 新签发的 refresh_token（旧的已作废，请保存新值） |
| token_type | string | 固定为 `Bearer` |
| expires_in | integer | 新 access_token 有效期（秒），默认 7200 |
| refresh_expires_in | integer | 新 refresh_token 有效期（秒），默认 2592000（自刷新时刻重新计算） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "3b7e9d2f5a1c8e4b0d6f3a9c2e5b8d1f4a7c0e3b6d9f2a5c8e1b4d7f0a3c6e9b",
    "token_type": "Bearer",
    "expires_in": 7200,
    "refresh_expires_in": 2592000
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | refresh_token 为空 / 不存在 / 已过期 / 已吊销（重用时还会吊销该用户全部会话） |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/auth/refresh' \
  -H 'Content-Type: application/json' \
  -d '{"refresh_token": "9f2c7a1e4b8d3f6a0c5e9d2b7a4f1c8e3d6b9a0f5e2c7d4b1a8f3e6c9d0b5a2f"}'
```

> 前端建议：access_token 过期（401 / 10001）时自动调用本接口续期并重放原请求；刷新失败则跳转登录。并发请求触发的多次刷新需注意保存最新 refresh_token，避免旧值重用触发全会话吊销。