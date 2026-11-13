use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
    pub email_verified_at: Option<NaiveDateTime>,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
use sqlx::MySqlPool;

pub async fn count(pool: &MySqlPool) -> Result<u64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, u64>("SELECT CAST(COUNT(*) AS UNSIGNED) FROM users")
        .fetch_one(pool)
        .await?;
    Ok(count)
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

pub async fn find_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
        .bind(email)
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
    email: Option<&str>,
    password_hash: &str,
    display_name: Option<&str>,
    role: &str,
    email_verified: bool,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO users (username, email, password_hash, display_name, role, email_verified_at) \
         VALUES (?, ?, ?, ?, ?, IF(?, UTC_TIMESTAMP(3), NULL))",
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(role)
    .bind(email_verified)
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
