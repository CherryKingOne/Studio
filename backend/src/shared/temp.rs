use std::path::{Path, PathBuf};

use crate::runtime::error::AppError;

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub async fn create(base_dir: &str, name: &str) -> Result<Self, AppError> {
        let path = Path::new(base_dir).join(format!("{name}-{}", uuid::Uuid::now_v7().simple()));
        tokio::fs::create_dir_all(&path)
            .await
            .map_err(|error| AppError::AudioProcess(error.to_string()))?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
