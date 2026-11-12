use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use bytes::Bytes;

use crate::config::RustFsSection;
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Storage {
    client: Client,
    bucket: String,
}

impl Storage {
    pub fn new(config: &RustFsSection) -> Self {
        let credentials = Credentials::new(
            config.access_key.clone(),
            config.secret_key.clone(),
            None,
            None,
            "novelcast",
        );
        let sdk_config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();
        Self {
            client: Client::from_conf(sdk_config),
            bucket: config.bucket.clone(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub async fn ensure_bucket(&self) -> Result<(), AppError> {
        if self
            .client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .is_ok()
        {
            return Ok(());
        }

        if let Err(error) = self
            .client
            .create_bucket()
            .bucket(&self.bucket)
            .send()
            .await
        {
            if self
                .client
                .head_bucket()
                .bucket(&self.bucket)
                .send()
                .await
                .is_err()
            {
                return Err(AppError::Storage(format!(
                    "ensure bucket {} failed: {}",
                    self.bucket, error
                )));
            }
        }
        Ok(())
    }

    pub async fn health(&self) -> Result<(), AppError> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    pub async fn put(&self, key: &str, bytes: Bytes, mime: &str) -> Result<(), AppError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(mime)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map(|_| ())
            .map_err(|e| AppError::Storage(format!("put object {key} failed: {e}")))
    }

    pub async fn get(&self, key: &str) -> Result<Bytes, AppError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Storage(format!("get object {key} failed: {e}")))?;
        let data = output
            .body
            .collect()
            .await
            .map_err(|e| AppError::Storage(format!("read object {key} failed: {e}")))?;
        Ok(data.into_bytes())
    }

    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| AppError::Storage(format!("delete object {key} failed: {e}")))
    }
}
