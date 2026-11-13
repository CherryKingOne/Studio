use crate::delivery::files::model::FileRecord;
use crate::delivery::files::repo::{self, NewFile};
use crate::runtime::error::AppError;
use crate::runtime::state::AppState;

pub async fn register(state: &AppState, file: NewFile) -> Result<u64, AppError> {
    Ok(repo::insert(&state.db, &file).await?)
}

pub async fn get(state: &AppState, id: u64) -> Result<FileRecord, AppError> {
    repo::find_by_id(&state.db, id)
        .await?
        .ok_or(AppError::NotFound { resource: "文件" })
}
