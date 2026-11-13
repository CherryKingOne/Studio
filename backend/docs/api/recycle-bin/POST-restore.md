# 恢复回收站条目

把回收站中的单本书籍恢复为正常状态：`deleted_at`、`deleted_by` 置空，`status` 恢复为删除前的值。章节、段落、角色随书籍一起恢复。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/recycle-bin/{resource_type}/{id}/restore` |
| 认证 | Bearer Token |
| Content-Type | `application/json`（可选） |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| resource_type | string | 是 | 资源类型，当前只支持 `book` |
| id | integer | 是 | 书籍 ID |

### Body 参数（可选）

不传请求体时按原书名、`on_conflict=fail` 恢复。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| title | string | 否 | 原书名 | 恢复时使用的新书名，1–255 字符 |
| on_conflict | string | 否 | `fail` | 书名冲突策略：`fail` 返回 `409`；`rename` 自动追加“（恢复）”或“（恢复 N）” |

冲突规则：

| 场景 | 行为 |
| --- | --- |
| 原书名未被占用 | 使用原书名恢复 |
| 指定 `title` 且被占用 | `409/20004`，不自动改名 |
| 未指定 `title`、原书名被占用、`on_conflict=fail` | `409/20004` |
| 未指定 `title`、原书名被占用、`on_conflict=rename` | 自动生成不冲突的书名 |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| resource_type | string | 资源类型，固定为 `book` |
| id | integer | 书籍 ID |
| book_id | integer | 书籍 ID（与 `id` 相同，便于跳转详情） |
| title | string | 恢复后的书名（可能被自动改名） |
| status | string | 恢复后的书籍状态 |
| restored_at | string | 恢复时间（ISO 8601 UTC） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "resource_type": "book",
    "id": 12,
    "book_id": 12,
    "title": "山海拾遗（恢复）",
    "status": "ready",
    "restored_at": "2026-11-13T09:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | `resource_type` 不是 `book`；`title` 为空或超过 255 字符；`on_conflict` 非法 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已不在回收站 |
| 409 | 20004 | 书名冲突且未使用 `rename` |

## curl 示例

按原书名恢复：

```bash
curl -X POST 'http://localhost:60181/api/v1/recycle-bin/book/12/restore' \
  -H 'Authorization: Bearer <access_token>'
```

书名冲突时自动改名：

```bash
curl -X POST 'http://localhost:60181/api/v1/recycle-bin/book/12/restore' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"on_conflict":"rename"}'
```

指定新书名：

```bash
curl -X POST 'http://localhost:60181/api/v1/recycle-bin/book/12/restore' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"title":"山海拾遗（修订版）"}'
```
