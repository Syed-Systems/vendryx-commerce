//! Provider `stheno-file-s3` for `stheno-file`

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use stheno_file::{FileProvider, FileUploadResult, PresignedUrlResult};
use stheno_types::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub cdn_url: Option<String>,
}

pub struct S3FileProvider {
    config: S3Config,
    in_memory_storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl S3FileProvider {
    pub fn new(config: S3Config) -> Self {
        Self {
            config,
            in_memory_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn config(&self) -> &S3Config {
        &self.config
    }

    fn build_public_url(&self, key: &str) -> String {
        if let Some(ref cdn) = self.config.cdn_url {
            format!("{}/{}", cdn.trim_end_matches('/'), key)
        } else if let Some(ref ep) = self.config.endpoint {
            format!(
                "{}/{}/{}",
                ep.trim_end_matches('/'),
                self.config.bucket,
                key
            )
        } else {
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.config.bucket, self.config.region, key
            )
        }
    }
}

#[async_trait]
impl FileProvider for S3FileProvider {
    fn provider_id(&self) -> &'static str {
        "s3"
    }

    async fn upload(
        &self,
        file_id: &str,
        filename: &str,
        _content_type: &str,
        bytes: &[u8],
        is_private: bool,
    ) -> Result<FileUploadResult> {
        let size_bytes = bytes.len() as i64;
        let extension = filename.rsplit('.').next().unwrap_or("bin");
        let prefix = if is_private { "private" } else { "uploads" };
        let key = format!("{}/{}.{}", prefix, file_id, extension);

        let mut map = self.in_memory_storage.write().unwrap();
        map.insert(key.clone(), bytes.to_vec());

        let url = self.build_public_url(&key);

        Ok(FileUploadResult { url, size_bytes })
    }

    async fn delete(&self, file_id: &str) -> Result<()> {
        let mut map = self.in_memory_storage.write().unwrap();
        let keys_to_remove: Vec<String> = map
            .keys()
            .filter(|k| k.contains(file_id))
            .cloned()
            .collect();
        for k in keys_to_remove {
            map.remove(&k);
        }
        Ok(())
    }

    async fn get_url(&self, file_id: &str) -> Result<String> {
        let map = self.in_memory_storage.read().unwrap();
        let key = map
            .keys()
            .find(|k| k.contains(file_id))
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: "file",
                id: file_id.to_string(),
            })?;
        Ok(self.build_public_url(&key))
    }

    async fn get_presigned_upload_url(
        &self,
        file_id: &str,
        filename: &str,
        content_type: &str,
        expires_in_seconds: u64,
        is_private: bool,
    ) -> Result<PresignedUrlResult> {
        let extension = filename.rsplit('.').next().unwrap_or("bin");
        let prefix = if is_private { "private" } else { "uploads" };
        let key = format!("{}/{}.{}", prefix, file_id, extension);
        let file_url = self.build_public_url(&key);

        let s3_endpoint = self
            .config
            .endpoint
            .as_deref()
            .unwrap_or("s3.amazonaws.com");
        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

        let upload_url = format!(
            "https://{}.{}/{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders=content-type%3Bhost&X-Amz-Signature=mock_signature_{}",
            self.config.bucket,
            s3_endpoint.trim_start_matches("https://").trim_start_matches("http://"),
            key,
            self.config.access_key_id,
            Utc::now().format("%Y%m%d"),
            self.config.region,
            timestamp,
            expires_in_seconds,
            file_id
        );

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert(
            "Host".to_string(),
            format!(
                "{}.{}",
                self.config.bucket,
                s3_endpoint
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
            ),
        );

        let expires_at = Utc::now() + Duration::seconds(expires_in_seconds as i64);

        Ok(PresignedUrlResult {
            upload_url,
            file_url,
            headers,
            expires_at,
        })
    }
    
    async fn get_presigned_download_url(
        &self,
        file_id: &str,
        expires_in_seconds: u64,
    ) -> Result<String> {
        let map = self.in_memory_storage.read().unwrap();
        let key = map
            .keys()
            .find(|k| k.contains(file_id))
            .cloned()
            .ok_or_else(|| Error::NotFound {
                entity: "file",
                id: file_id.to_string(),
            })?;
            
        let s3_endpoint = self
            .config
            .endpoint
            .as_deref()
            .unwrap_or("s3.amazonaws.com");
        let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

        let download_url = format!(
            "https://{}.{}/{}?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}%2F{}%2F{}%2Fs3%2Faws4_request&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-Signature=mock_signature_{}",
            self.config.bucket,
            s3_endpoint.trim_start_matches("https://").trim_start_matches("http://"),
            key,
            self.config.access_key_id,
            Utc::now().format("%Y%m%d"),
            self.config.region,
            timestamp,
            expires_in_seconds,
            file_id
        );

        Ok(download_url)
    }
}
