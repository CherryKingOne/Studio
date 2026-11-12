use sqlx::MySqlPool;

use crate::features::auth::model::User;

pub async fn count(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count as u64)
}

pub async fn find_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_id(pool: &MySqlPool, id: u64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert(
    pool: &MySqlPool,
    username: &str,
    password_hash: &str,
    display_name: Option<&str>,
    role: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, display_name, role) VALUES (?, ?, ?, ?)",
    )
    .bind(username)
    .bind(password_hash)
    .bind(display_name)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id())
}

pub async fn update_last_login(pool: &MySqlPool, id: u64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_login_at = UTC_TIMESTAMP(3) WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_password(
    pool: &MySqlPool,
    id: u64,
    password_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
