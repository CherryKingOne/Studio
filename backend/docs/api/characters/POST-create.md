# 创建角色

为书籍手动创建角色。角色名书内唯一，创建来源标记为 `manual`。创建后可将角色绑定默认音色，并在段落上指派为说话人。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/books/{book_id}/characters` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| book_id | integer | 是 | 书籍 ID |

### Body 参数（JSON）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| name | string | 是 | - | 角色名，去除首尾空白后 1~64 个字符；同书重名返回 `20004` |
| aliases | array[string] | 否 | [] | 别名列表，最多 20 个；元素自动去除首尾空白并过滤空串 |
| gender | string | 否 | unknown | 性别：`male` / `female` / `unknown`（自动转小写） |
| age_group | string | 否 | unknown | 年龄段：`child` / `teen` / `adult` / `elder` / `unknown`（自动转小写） |
| description | string | 否 | null | 角色描述 |
| default_voice_id | integer | 否 | null | 角色默认音色 ID（对白段落未指定音色时优先使用） |
| color | string | 否 | null | 前端展示颜色，如 `#FF6600` |

## 响应参数

HTTP `201 Created`，统一包装结构（`message` 为 `created`），`data` 字段（CharacterDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 角色 ID |
| name | string | 角色名 |
| aliases | array[string] | 别名列表 |
| gender | string | 性别：`male` / `female` / `unknown` |
| age_group | string | 年龄段：`child` / `teen` / `adult` / `elder` / `unknown` |
| description | string / null | 角色描述 |
| default_voice_id | integer / null | 角色默认音色 ID |
| color | string / null | 展示颜色 |
| source | string | 来源，手动创建固定为 `manual` |
| segment_count | integer | 关联段落数，新建为 0 |

## 响应示例

```json
{
  "code": 0,
  "message": "created",
  "data": {
    "id": 3,
    "name": "林少侠",
    "aliases": ["小林", "少侠"],
    "gender": "male",
    "age_group": "teen",
    "description": "主角，少年剑客",
    "default_voice_id": 8,
    "color": "#FF6600",
    "source": "manual",
    "segment_count": 0
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 角色名为空 / 超 64 字符；别名超过 20 个；gender / age_group 取值非法 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在 |
| 409 | 20004 | 同书角色名已存在 |

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/books/1/characters' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "林少侠",
    "aliases": ["小林", "少侠"],
    "gender": "male",
    "age_group": "teen",
    "description": "主角，少年剑客",
    "default_voice_id": 8,
    "color": "#FF6600"
  }'
```