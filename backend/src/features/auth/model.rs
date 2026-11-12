use chrono::NaiveDateTime;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub role: String,
    pub last_login_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
