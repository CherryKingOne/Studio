# 回收站汇总

返回回收站条目总数、按资源类型的计数、保留期配置和到期提示，用于回收站入口角标与保留期提示。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `GET` |
| 请求路径 | `/api/v1/recycle-bin/summary` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

无。

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| total | integer | 回收站条目总数 |
| resource_types | array | 按资源类型计数，当前只有 `book` |
| retention_days | integer | 配置的保留天数；`0` 表示永久保留 |
| oldest_deleted_at | string / null | 最早删除时间；空回收站为 `null` |
| next_expire_at | string / null | 最早到期时间；空回收站或保留期为 0 时为 `null` |

`resource_types[]` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| resource_type | string | 资源类型，当前固定为 `book` |
| count | integer | 该类型条目数 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "total": 3,
    "resource_types": [
      { "resource_type": "book", "count": 3 }
    ],
    "retention_days": 30,
    "oldest_deleted_at": "2026-11-01T08:00:00Z",
    "next_expire_at": "2026-12-01T08:00:00Z"
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

空回收站：

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "total": 0,
    "resource_types": [
      { "resource_type": "book", "count": 0 }
    ],
    "retention_days": 30,
    "oldest_deleted_at": null,
    "next_expire_at": null
  },
  "request_id": "..."
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

## curl 示例

```bash
curl -X GET 'http://localhost:60181/api/v1/recycle-bin/summary' \
  -H 'Authorization: Bearer <access_token>'
```
