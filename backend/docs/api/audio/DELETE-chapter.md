# 删除章节音频

清除章节的整章拼接音频关联（不删除段落音频），章节状态按段落合成情况回退为 `tts_done`（全部段落已合成）或 `tts_partial`（存在未完成段落）。成功后返回 `204`，无响应体。

## 基本信息

| 项目 | 内容 |
| --- | --- |
| 请求方法 | `DELETE` |
| 请求路径 | `/api/v1/chapters/{id}/audio` |
| 认证 | Bearer Token |
| Content-Type | 无请求体 |

## 请求参数

### 路径参数

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| id | integer | 是 | 章节 ID |

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
| 404 | 20001 | 章节不存在，或章节没有拼接音频（`message` 为「章节音频 不存在」） |

## curl 示例

```bash
curl -i -X DELETE 'http://localhost:60181/api/v1/chapters/10/audio' \
  -H 'Authorization: Bearer <access_token>'
```

> 删除后可重新调用 `POST /api/v1/chapters/{id}/merge` 生成新的整章音频。