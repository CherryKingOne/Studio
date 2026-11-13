# 角色列表

查询某本书籍的全部角色（不分页），附带每个角色关联的段落数。角色用于对白说话人指派与默认音色绑定。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/books/{book_id}/characters` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| book_id | integer | 是 | 书籍 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 角色列表，元素结构见下表 |
| total | integer | 角色总数 |

`items[]` 元素字段（CharacterDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 角色 ID |
| name | string | 角色名（书内唯一） |
| aliases | array[string] | 别名列表（用于识别匹配），可为空数组 |
| gender | string | 性别：`male` / `female` / `unknown` |
| age_group | string | 年龄段：`child` / `teen` / `adult` / `elder` / `unknown` |
| description | string / null | 角色描述 |
| default_voice_id | integer / null | 角色默认音色 ID，未绑定为 `null` |
| color | string / null | 前端展示颜色（如 `#FF6600`），未设置为 `null` |
| source | string | 来源：`manual`（手动创建）/ `rule`（规则自动识别） |
| segment_count | integer | 已指派该角色为说话人的段落数 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 3,
        "name": "林少侠",
        "aliases": ["小林", "少侠"],
        "gender": "male",
        "age_group": "teen",
        "description": "主角，少年剑客",
        "default_voice_id": 8,
        "color": "#FF6600",
        "source": "manual",
        "segment_count": 42
      }
    ],
    "total": 1
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

> 书籍 ID 不存在时返回空列表（`items: []`、`total: 0`），不返回 404。

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/books/1/characters' \
  -H 'Authorization: Bearer <access_token>'
```