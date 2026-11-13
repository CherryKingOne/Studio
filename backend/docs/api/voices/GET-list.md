# 音色列表

分页查询音色库，支持按供应商、语言、性别、关键词、启用状态筛选。音色用于旁白与角色配音。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/voices` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| provider | string | 否 | - | 按供应商精确筛选：`openai` / `minimax` / `stepfun` |
| locale | string | 否 | - | 按语言精确筛选，如 `zh-CN`、`multi` |
| gender | string | 否 | - | 按性别精确筛选，如 `male` / `female` / `unknown` |
| keyword | string | 否 | - | 关键词，对显示名（display_name）或供应商音色 ID（provider_voice_id）模糊匹配 |
| enabled | boolean | 否 | - | 按启用状态筛选；不传则返回全部 |
| page | integer | 否 | 1 | 页码，从 1 开始 |
| page_size | integer | 否 | 50 | 每页条数，限制 1~200（注意默认值与其他列表不同） |

## 响应参数

HTTP `200`，统一包装结构，`data` 为分页对象：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 音色列表，元素结构见下表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 满足条件的音色总数 |

`items[]` 元素字段（VoiceDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 音色 ID（本系统内部 ID） |
| provider | string | 供应商：`openai` / `minimax` / `stepfun` |
| provider_voice_id | string | 供应商侧音色标识，如 `alloy` |
| display_name | string | 显示名称 |
| locale | string | 语言 / 地区，如 `zh-CN`、`multi` |
| gender | string | 性别：`male` / `female` / `unknown` |
| styles | array[string] | 支持的风格 / 情绪标签列表，可为空数组 |
| sample_url | string | 示例音频地址（`/api/v1/voices/{id}/sample`） |
| enabled | boolean | 是否启用（停用的音色不参与自动解析） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 5,
        "provider": "openai",
        "provider_voice_id": "alloy",
        "display_name": "Alloy",
        "locale": "multi",
        "gender": "unknown",
        "styles": [],
        "sample_url": "/api/v1/voices/5/sample",
        "enabled": true
      }
    ],
    "page": 1,
    "page_size": 50,
    "total": 1
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/voices?provider=openai&enabled=true&page=1&page_size=50' \
  -H 'Authorization: Bearer <access_token>'
```

按关键词搜索：

```bash
curl -X GET 'http://localhost:60181/api/v1/voices?keyword=alloy' \
  -H 'Authorization: Bearer <access_token>'
```