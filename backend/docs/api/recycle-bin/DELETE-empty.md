# 清空回收站

物理删除回收站中的全部书籍，不可恢复。必须显式传 `confirm=true`，防止误触。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `DELETE` |
| 请求路径 | `/api/v1/recycle-bin` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### Query 参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| confirm | boolean | 是 | 必须为 `true`，否则返回 `400/20002` |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| purged | integer | 成功彻底删除数量 |
| failed | integer | 失败数量（如存在进行中任务） |
| files_deleted | integer | 本次删除的 `files` 记录总数 |
| objects_failed | integer | 对象存储删除失败数（仅告警） |

## 响应示例

```json
{
  "code": 0,
  "message": "ok",
  "data": {
    "purged": 2,
    "failed": 0,
    "files_deleted": 8,
    "objects_failed": 0
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | 缺少 `confirm=true` |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |

## curl 示例

```bash
curl -X DELETE 'http://localhost:60181/api/v1/recycle-bin?confirm=true' \
  -H 'Authorization: Bearer <access_token>'
```
