# 彻底删除回收站条目

物理删除回收站中的单本书籍，不可恢复。同时删除该书籍的章节、段落、角色、导出记录、任务，以及未被其他数据引用的文件记录和对象存储文件。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `DELETE` |
| 请求路径 | `/api/v1/recycle-bin/{resource_type}/{id}` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| resource_type | string | 是 | 资源类型，当前只支持 `book` |
| id | integer | 是 | 书籍 ID |

## 响应参数

HTTP `200`，统一包装结构，`data` 字段：

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| resource_type | string | 资源类型，固定为 `book` |
| id | integer | 书籍 ID |
| purged | boolean | 固定为 `true`，表示彻底删除成功 |
| files_deleted | integer | 本次删除的 `files` 记录数 |
| objects_failed | integer | 对象存储删除失败数（仅告警，不影响 `purged=true`） |

## 响应示例

```json
{
  "code": 0,
  "message": "deleted",
  "data": {
    "resource_type": "book",
    "id": 12,
    "purged": true,
    "files_deleted": 4,
    "objects_failed": 0
  },
  "request_id": "01928f3a-7c4e-7b2a-9d1f-3e5a7c9b2d40"
}
```

## 错误码

| HTTP 状态码 | code | 触发条件 |
| --- | --- | --- |
| 400 | 20002 | `resource_type` 不是 `book` |
| 401 | 10001 | 未携带 Token 或 Token 无效 / 过期 |
| 404 | 20001 | 书籍不存在或已彻底删除 |
| 409 | 20003 | 书籍在正常列表中（未进回收站）；或存在进行中任务 |

## 说明

- 只允许删除 `deleted_at IS NOT NULL` 的书籍；对正常书籍返回 `409`；
- 存在 `queued` / `running` 任务时拒绝彻底删除；
- 文件清理是保守的：封面、导入源文件、章节/段落音频、导出 ZIP 只有在没有被其他书籍、章节、段落、音色、导出引用时才会删除；
- 对象存储删除失败只记录日志，不回滚数据库，也不改变 `purged=true` 结果，可通过响应的 `objects_failed` 观察失败数量。

## curl 示例

```bash
curl -i -X DELETE 'http://localhost:60181/api/v1/recycle-bin/book/12' \
  -H 'Authorization: Bearer <access_token>'
```
