use chrono::NaiveDateTime;
use sqlx::types::Json;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Character {
    pub id: u64,
    pub book_id: u64,
    pub name: String,
    pub aliases: Json<Vec<String>>,
    pub gender: String,
    pub age_group: String,
    pub description: Option<String>,
    pub default_voice_id: Option<u64>,
    pub color: Option<String>,
    pub source: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CharacterRow {
    #[sqlx(flatten)]
    pub character: Character,
    pub segment_count: u64,
}
