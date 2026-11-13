# 更新角色

部分更新角色信息。仅传入需要修改的字段，未传入的字段保持原值。

**副作用（重要）**：修改 `default_voice_id`（角色默认音色）时，所有已指派该角色且已合成完成的段落会重置为 `pending`，相关章节的整章拼接音频会被清除——需要重新合成与拼接。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `PATCH` |
| 请求路径 | `/api/v1/characters/{id}` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 角色 ID |

### Body 参数（JSON，全部可选）

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| name | string | 否 | 保持原值 | 角色名，1~64 个字符；改名后同书重名返回 `20004` |
| aliases | array[string] | 否 | 保持原值 | 别名列表，最多 20 个；**整体替换**，自动去空白过滤空串 |
| gender | string | 否 | 保持原值 | `male` / `female` / `unknown` |
| age_group | string | 否 | 保持原值 | `child` / `teen` / `adult` / `elder` / `unknown` |
| description | string | 否 | 保持原值 | 角色描述 |
| default_voice_id | integer | 否 | 保持原值 | 角色默认音色 ID；变更会使关联段落失效（本接口不支持清空为 null） |
| color | string | 否 | 保持原值 | 展示颜色 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为更新后的角色（CharacterDto）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 角色 ID |
| name | string | 角色名 |
| aliases | array[string] | 别名列表 |
| gender | string | 性别 |
| age_group | string | 年龄段 |
| description | string / null | 角色描述 |
| default_voice_id | integer / null | 角色默认音色 ID |
| color | string / null | 展示颜色 |
| source | string | 来源：`manual` / `rule` |
| segment_count | integer | 关联段落数 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "id": 3,
    "name": "林少侠",
    "aliases": ["小林", "少侠", "林兄弟"],
    "gender": "male",
    "age_group": "teen",
    "description": "主角，少年剑客",
    "default_voice_id": 9,
    "color": "#FF6600",
    "source": "manual",
    "segment_count": 42
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 角色名长度非法；别名超过 20 个；gender / age_group 取值非法 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 角色不存在 |
| 409 | 20004 | 改名后同书角色名已存在 |

## curl 示例

```bash
curl -X PATCH 'http://localhost:60181/api/v1/characters/3' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{
    "aliases": ["小林", "少侠", "林兄弟"],
    "default_voice_id": 9
  }'
```