# 书籍列表

分页查询书籍列表，支持按关键词、状态筛选。默认排除软删除（回收站）书籍，以及 `archived` 状态书籍，按创建时间倒序排列。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/books` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### Query 参数

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| page | integer | 否 | 1 | 页码，从 1 开始，小于 1 按 1 处理 |
| page_size | integer | 否 | 20 | 每页条数，限制 1~100 |
| keyword | string | 否 | - | 关键词，对书名（title）或作者（author）做模糊匹配（LIKE %keyword%） |
| status | string | 否 | - | 按状态精确筛选：`draft` / `importing` / `parsing` / `segmenting` / `ready` / `failed` / `archived` |
| include_archived | boolean | 否 | false | 是否包含已归档书籍；为 `false` 时排除 `archived` 状态 |

## 响应参数

HTTP `200`，统一包装结构，`data` 为分页对象：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| items | array | 书籍摘要列表，元素结构见下表 |
| page | integer | 当前页码 |
| page_size | integer | 每页条数 |
| total | integer | 满足条件的书籍总数 |

`items[]` 元素字段（BookSummary）：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| id | integer | 书籍 ID |
| title | string | 书名 |
| author | string / null | 作者 |
| cover_url | string / null | 封面访问地址（`/api/v1/files/{cover_file_id}`），未上传封面为 `null` |
| status | string | 书籍状态：`draft` / `importing` / `parsing` / `segmenting` / `ready` / `failed` / `archived` |
| total_chapters | integer | 章节总数 |
| total_segments | integer | 段落总数 |
| total_duration_ms | integer | 已拼接章节音频总时长（毫秒） |
| created_at | string | 创建时间（ISO 8601 UTC） |
| updated_at | string | 更新时间（ISO 8601 UTC） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "items": [
      {
        "id": 1,
        "title": "示例小说",
        "author": "佚名",
        "cover_url": "/api/v1/files/12",
        "status": "ready",
        "total_chapters": 20,
        "total_segments": 480,
        "total_duration_ms": 3600000,
        "created_at": "2026-11-13T08:00:00Z",
        "updated_at": "2026-11-13T09:00:00Z"
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
curl -X GET 'http://localhost:60181/api/v1/books?page=1&page_size=20&keyword=示例&status=ready' \
  -H 'Authorization: Bearer <access_token>'
```

包含已归档书籍：

```bash
curl -X GET 'http://localhost:60181/api/v1/books?include_archived=true' \
  -H 'Authorization: Bearer <access_token>'
```