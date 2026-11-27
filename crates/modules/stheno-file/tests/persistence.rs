//! File records survive the service instance that wrote them.

use async_trait::async_trait;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_file::{
    FileProvider, FileService, FileUploadResult, PresignedUrlResult, UploadFileInput,
};
use stheno_types::Result;

/// Provider that stores nothing: the file row in Postgres is the only record.
struct NullProvider;

#[async_trait]
impl FileProvider for NullProvider {
    fn provider_id(&self) -> &'static str {
        "null"
    }
    async fn upload(
        &self,
        file_id: &str,
        _filename: &str,
        _content_type: &str,
        bytes: &[u8],
        _is_private: bool,
    ) -> Result<FileUploadResult> {
        Ok(FileUploadResult {
            url: format!("https://cdn.test/{file_id}"),
            size_bytes: bytes.len() as i64,
        })
    }
    async fn delete(&self, _file_id: &str) -> Result<()> {
        Ok(())
    }
    async fn get_url(&self, file_id: &str) -> Result<String> {
        Ok(format!("https://cdn.test/{file_id}"))
    }
    async fn get_presigned_upload_url(
        &self,
        _file_id: &str,
        _filename: &str,
        _content_type: &str,
        _expires_in_seconds: u64,
        _is_private: bool,
    ) -> Result<PresignedUrlResult> {
        unreachable!("not used by this test")
    }
    async fn get_presigned_download_url(
        &self,
        _file_id: &str,
        _expires_in_seconds: u64,
    ) -> Result<String> {
        unreachable!("not used by this test")
    }
}

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = FileService::new(std::sync::Arc::new(NullProvider), pool.clone());
    let png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
    let written = writer
        .upload_file(UploadFileInput {
            filename: "outlive.png".to_string(),
            content_type: Some("image/png".to_string()),
            bytes: png.clone(),
            metadata: Some(serde_json::json!({ "alt": "kept" })),
            is_private: false,
        })
        .await
        .expect("upload");
    drop(writer);

    let reader = FileService::new(std::sync::Arc::new(NullProvider), pool);
    let fetched = reader.get_file(&written.id).await.expect("get");
    assert_eq!(fetched.id, written.id);
    assert_eq!(fetched.filename, "outlive.png");
    assert_eq!(fetched.mime_type, "image/png");
    assert_eq!(fetched.size_bytes, png.len() as i64);
    assert_eq!(fetched.metadata, Some(serde_json::json!({ "alt": "kept" })));

    db.drop().await;
}
