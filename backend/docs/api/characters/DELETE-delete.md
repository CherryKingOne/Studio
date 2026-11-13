# 删除角色

删除角色，并解除所有段落对该角色的说话人指派。成功后返回 `204`，无响应体。

**副作用**：被解除指派的段落中已合成完成（`done`）的会重置为 `pending`，相关章节的整章拼接音频会被清除——需要重新合成与拼接。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `DELETE` |
| 请求路径 | `/api/v1/characters/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 角色 ID |

## 响应参数

HTTP `204 No Content`，**无响应体**。

## 响应示例

```
HTTP/1.1 204 No Content
x-request-id: 01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 角色不存在 |

## curl 示例

```bash
curl -i -X DELETE 'http://localhost:60181/api/v1/characters/3' \
  -H 'Authorization: Bearer <access_token>'
```