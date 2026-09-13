//! Provider `stheno-file-local` for `stheno-file`

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use stheno_file::{FileProvider, FileUploadResult, PresignedUrlResult};
use stheno_types::{Error, Result};

pub struct LocalFileProvider {
    base_dir: PathBuf,
    base_url: String,
    // In-memory backing buffer for fast tests / fallback
    in_memory_storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl LocalFileProvider {
    pub fn new(base_dir: impl Into<PathBuf>, base_url: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            in_memory_storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn base_dir(&self) -> &std::path::Path {
        &self.base_dir
    }
}

#[async_trait]
impl FileProvider for LocalFileProvider {
    fn provider_id(&self) -> &'static str {
        "local"
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
        let mut map = self.in_memory_storage.write().unwrap();
        map.insert(file_id.to_string(), bytes.to_vec());

        let extension = filename.rsplit('.').next().unwrap_or("bin");
        let prefix = if is_private { "private" } else { "uploads" };
        let url = format!("{}/{}/{}.{}", self.base_url, prefix, file_id, extension);

        Ok(FileUploadResult { url, size_bytes })
    }

    async fn delete(&self, file_id: &str) -> Result<()> {
        let mut map = self.in_memory_storage.write().unwrap();
        map.remove(file_id);
        Ok(())
    }

    async fn get_url(&self, file_id: &str) -> Result<String> {
        let map = self.in_memory_storage.read().unwrap();
        if !map.contains_key(file_id) {
            return Err(Error::NotFound {
                entity: "file",
                id: file_id.to_string(),
            });
        }
        Ok(format!("{}/uploads/{}", self.base_url, file_id))
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
        let upload_url = format!("{}/api/upload/direct/{}", self.base_url, file_id);
        let file_url = format!("{}/{}/{}.{}", self.base_url, prefix, file_id, extension);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("X-File-Id".to_string(), file_id.to_string());

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
        if !map.contains_key(file_id) {
            return Err(Error::NotFound {
                entity: "file",
                id: file_id.to_string(),
            });
        }
        
        let download_url = format!("{}/api/download/direct/{}?expires={}", self.base_url, file_id, expires_in_seconds);
        Ok(download_url)
    }
}
