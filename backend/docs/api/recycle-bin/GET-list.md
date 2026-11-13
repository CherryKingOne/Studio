# 回收站列表

分页查询已删除（进入回收站）的书籍，按删除时间倒序排列。当前只管理书籍；章节、段落、角色随书籍整体进入回收站。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/recycle-bin` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| page | integer | 否 | 1 | 页码，从 1 开始，小于 1 按 1 处理 |
| page_size | integer | 否 | 20 | 每页条数，限制 1~100 |
| keyword | string | 否 | - | 关键词，对书名（title）或作者（author）做模糊匹配 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为分页对象：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 回收站条目列表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 回收站条目总数 |

`items[]` 元素字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| resource_type | string | 资源类型，当前固定为 `book` |
| id | integer | 书籍 ID |
| title | string | 书名（删除时的书名） |
| author | string / null | 作者 |
| cover_url | string / null | 封面地址（`/api/v1/files/{cover_file_id}`） |
| status | string | 删除前的书籍状态（`draft` / `ready` / ...），不因删除改变 |
| total_chapters | integer | 章节数 |
| total_segments | integer | 段落数 |
| total_duration_ms | integer | 已拼接章节音频总时长（毫秒） |
| deleted_at | string | 删除时间（ISO 8601 UTC） |
| deleted_by | object / null | 删除人；历史数据或用户已注销时为 `null` |
| expires_at | string / null | 到期时间（`deleted_at + 保留天数`）；保留期为 0 时为 `null` |
| remaining_days | integer / null | 剩余天数，向上取整；已到期为 `0`；保留期为 0 时为 `null` |

`deleted_by` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 用户 ID |
| username | string | 用户名 |
| display_name | string / null | 显示名 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "resource_type": "book",
        "id": 12,
        "title": "山海拾遗",
        "author": "某某",
        "cover_url": "/api/v1/files/88",
        "status": "ready",
        "total_chapters": 36,
        "total_segments": 842,
        "total_duration_ms": 5400000,
        "deleted_at": "2026-11-13T08:00:00Z",
        "deleted_by": {
          "id": 1,
          "username": "admin",
          "display_name": "管理员"
        },
        "expires_at": "2026-12-13T08:00:00Z",
        "remaining_days": 30
      }
    ],
    "page": 1,
    "page_size": 20,
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
curl -X GET 'http://localhost:60181/api/v1/recycle-bin?page=1&page_size=20&keyword=山海' \
  -H 'Authorization: Bearer <access_token>'
```
