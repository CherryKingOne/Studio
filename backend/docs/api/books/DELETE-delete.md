# 删除书籍

软删除书籍：写入 `deleted_at`（UTC）与 `deleted_by`（当前用户），`status` 保持不变，数据保留并进入回收站。存在进行中任务（排队 / 执行中）时禁止删除。成功后返回 `204`，无响应体。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `DELETE` |
| 请求路径 | `/api/v1/books/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 书籍 ID |

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
| 404 | 20001 | 书籍不存在或已软删除 |
| 409 | 20003 | 书籍存在进行中的任务，暂不能删除 |

## curl 示例

```bash
curl -i -X DELETE 'http://localhost:60181/api/v1/books/1' \
  -H 'Authorization: Bearer <access_token>'
```

> 软删除后书籍不再出现在默认列表，也无法通过详情/章节/段落/角色接口访问；请通过 `GET /api/v1/recycle-bin` 查看，使用 `POST /api/v1/recycle-bin/book/{id}/restore` 恢复，或 `DELETE /api/v1/recycle-bin/book/{id}` 彻底删除。