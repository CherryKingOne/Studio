use serde::{Deserialize, Serialize};

use crate::casting::characters::types::{Character, CharacterRow};

#[derive(Debug, Serialize)]
pub struct CharacterDto {
    pub id: u64,
    pub name: String,
    pub aliases: Vec<String>,
    pub gender: String,
    pub age_group: String,
    pub description: Option<String>,
    pub default_voice_id: Option<u64>,
    pub color: Option<String>,
    pub source: String,
    pub segment_count: u64,
}

impl From<CharacterRow> for CharacterDto {
    fn from(row: CharacterRow) -> Self {
        Self::from_character(&row.character, row.segment_count)
    }
}

impl From<Character> for CharacterDto {
    fn from(character: Character) -> Self {
        Self::from_character(&character, 0)
    }
}

impl CharacterDto {
    fn from_character(character: &Character, segment_count: u64) -> Self {
        Self {
            id: character.id,
            name: character.name.clone(),
            aliases: character.aliases.0.clone(),
            gender: character.gender.clone(),
            age_group: character.age_group.clone(),
            description: character.description.clone(),
            default_voice_id: character.default_voice_id,
            color: character.color.clone(),
            source: character.source.clone(),
            segment_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CharacterListResponse {
    pub items: Vec<CharacterDto>,
    pub total: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCharacterRequest {
    pub name: String,
    pub aliases: Option<Vec<String>>,
    pub gender: Option<String>,
    pub age_group: Option<String>,
    pub description: Option<String>,
    pub default_voice_id: Option<u64>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCharacterRequest {
    pub name: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub gender: Option<String>,
    pub age_group: Option<String>,
    pub description: Option<String>,
    pub default_voice_id: Option<u64>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DetectRequest {
    #[serde(default)]
    pub overwrite_existing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectPayload {
    pub book_id: u64,
    pub overwrite_existing: bool,
}

#[derive(Debug, Serialize)]
pub struct DetectResponse {
    pub job_id: u64,
}
