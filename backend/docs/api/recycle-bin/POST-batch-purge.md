# 批量彻底删除回收站条目

批量物理删除回收站中的书籍。单个条目失败不会阻断其他条目，整体返回 `200`。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `POST` |
| 请求路径 | `/api/v1/recycle-bin/purge` |
| 认证 | Bearer Token |
| Content-Type | `application/json` |

## 请求参数

### Body 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| items | array | 是 | 待彻底删除条目，1–100 条，不能重复 |
| items[].resource_type | string | 是 | 资源类型，当前只支持 `book` |
| items[].id | integer | 是 | 书籍 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| purged | integer | 成功彻底删除数量 |
| failed | integer | 失败数量 |
| files_deleted | integer | 本次删除的 `files` 记录总数 |
| objects_failed | integer | 对象存储删除失败数（仅告警，不影响 `purged`） |
| items | array | 逐项结果，顺序与请求一致 |

`items[]` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| resource_type | string | 资源类型 |
| id | integer | 书籍 ID |
| success | boolean | 是否成功 |
| title | null | 彻底删除时固定为 `null` |
| code | integer / null | 失败时的业务错误码 |
| message | string / null | 失败时的错误描述 |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "purged": 1,
    "failed": 1,
    "files_deleted": 4,
    "objects_failed": 0,
    "items": [
      {
        "resource_type": "book",
        "id": 12,
        "success": true,
        "title": null,
        "code": null,
        "message": null
      },
      {
        "resource_type": "book",
        "id": 13,
        "success": false,
        "title": null,
        "code": 20003,
        "message": "书籍不在回收站中，无法彻底删除"
      }
    ]
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | `items` 为空或超过 100 条；存在重复 ID；`resource_type` 非法 |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

> 单个条目的失败（`404/409`）体现在 `items[].code/message` 中，不影响其他条目。

## curl 示例

```bash
curl -X POST 'http://localhost:60181/api/v1/recycle-bin/purge' \
  -H 'Authorization: Bearer <access_token>' \
  -H 'Content-Type: application/json' \
  -d '{"items":[{"resource_type":"book","id":12}]}'
```
