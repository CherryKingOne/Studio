use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportPayload {
    pub book_id: u64,
    pub file_id: u64,
    pub max_chars: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub book_id: u64,
    pub job_id: u64,
}
