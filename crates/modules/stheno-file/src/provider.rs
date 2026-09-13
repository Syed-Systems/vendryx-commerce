use async_trait::async_trait;
use stheno_types::Result;

#[derive(Debug, Clone)]
pub struct FileUploadResult {
    pub url: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone)]
pub struct PresignedUrlResult {
    pub upload_url: String,
    pub file_url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait FileProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn upload(
        &self,
        file_id: &str,
        filename: &str,
        content_type: &str,
        bytes: &[u8],
        is_private: bool,
    ) -> Result<FileUploadResult>;
    async fn delete(&self, file_id: &str) -> Result<()>;
    async fn get_url(&self, file_id: &str) -> Result<String>;
    async fn get_presigned_upload_url(
        &self,
        file_id: &str,
        filename: &str,
        content_type: &str,
        expires_in_seconds: u64,
        is_private: bool,
    ) -> Result<PresignedUrlResult>;
    async fn get_presigned_download_url(
        &self,
        file_id: &str,
        expires_in_seconds: u64,
    ) -> Result<String>;
}
