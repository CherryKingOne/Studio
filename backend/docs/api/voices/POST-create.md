# 手动登记音色

手动登记一个供应商音色（适用于供应商不提供音色列表接口的情况）。按 `(provider, provider_voice_id)` **Upsert**：已存在则更新显示名等信息，不存在则新建。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/voices` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| provider | string | 是 | - | 供应商，仅支持 `openai` / `minimax` / `stepfun`（自动转小写） |
| provider_voice_id | string | 是 | - | 供应商侧音色标识（如 `alloy`、`female-shaonv`），去除首尾空白后不能为空 |
| display_name | string | 是 | - | 显示名称，去除首尾空白后不能为空 |
| locale | string | 否 | multi | 语言 / 地区，如 `zh-CN`；空白按默认值处理 |
| gender | string | 否 | unknown | 性别：`male` / `female` / `unknown`；空白按默认值处理 |
| styles | array[string] | 否 | [] | 支持的风格 / 情绪标签列表 |

## 响应参数

HTTP `201 Created`，统一包装结构（`message` 为 `created`），`data` 字段（VoiceDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 音色 ID |
| provider | string | 供应商 |
| provider_voice_id | string | 供应商侧音色标识 |
| display_name | string | 显示名称 |
| locale | string | 语言 / 地区 |
| gender | string | 性别 |
| styles | array[string] | 风格标签列表 |
| sample_url | string | 示例音频地址（`/api/v1/voices/{id}/sample`） |
| enabled | boolean | 是否启用 |

## 响应示例

```json
{
  "code": 0,
  "message": "created",
  "data": {
    "id": 8,
    "provider": "minimax",
    "provider_voice_id": "female-shaonv",
    "display_name": "少女音",
    "locale": "zh-CN",
    "gender": "female",
    "styles": ["happy", "sad"],
    "sample_url": "/api/v1/voices/8/sample",
    "enabled": true
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | provider 不支持；provider_voice_id / display_name 为空 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/voices' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "provider": "minimax",
    "provider_voice_id": "female-shaonv",
    "display_name": "少女音",
    "locale": "zh-CN",
    "gender": "female",
    "styles": ["happy", "sad"]
  }'
```