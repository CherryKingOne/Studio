use sqlx::MySqlPool;

use crate::delivery::files::model::FileRecord;

pub struct NewFile {
    pub bucket: String,
    pub object_key: String,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub kind: String,
    pub duration_ms: Option<u32>,
}

pub async fn insert(pool: &MySqlPool, file: &NewFile) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO files (bucket, object_key, file_name, mime, size_bytes, sha256, kind, duration_ms)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&file.bucket)
    .bind(&file.object_key)
    .bind(&file.file_name)
    .bind(&file.mime)
    .bind(file.size_bytes)
    .bind(&file.sha256)
    .bind(&file.kind)
    .bind(file.duration_ms)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<FileRecord>, sqlx::Error> {
    sqlx::query_as::<_, FileRecord>("SELECT * FROM files WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
