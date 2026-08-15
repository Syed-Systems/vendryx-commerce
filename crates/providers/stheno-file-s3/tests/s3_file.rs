use std::sync::Arc;
use stheno_file::{FileService, UploadFileInput};
use stheno_file_s3::{S3Config, S3FileProvider};

#[tokio::test]
async fn test_s3_file_provider_upload_and_presigned_urls() {
    let config = S3Config {
        bucket: "stheno-media".to_string(),
        region: "us-east-1".to_string(),
        endpoint: None,
        access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
        secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        cdn_url: Some("https://cdn.sthenocommerce.com".to_string()),
    };

    let provider = Arc::new(S3FileProvider::new(config));
    let service = FileService::new(provider);

    // Upload WebP file
    // RIFF (4 bytes) + len (4 bytes) + WEBP (4 bytes)
    let webp_bytes = vec![
        b'R', b'I', b'F', b'F', 0x00, 0x00, 0x00, 0x04, b'W', b'E', b'B', b'P', 0x01, 0x02,
    ];

    let file = service
        .upload_file(UploadFileInput {
            filename: "hero_banner.webp".to_string(),
            content_type: Some("image/webp".to_string()),
            bytes: webp_bytes,
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(file.filename, "hero_banner.webp");
    assert_eq!(file.mime_type, "image/webp");
    assert!(file
        .url
        .starts_with("https://cdn.sthenocommerce.com/uploads/"));
    assert!(file.url.ends_with(".webp"));

    // Presigned Upload
    let presigned = service
        .create_presigned_upload(stheno_file::CreatePresignedUploadInput {
            filename: "spec.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            expires_in_seconds: Some(1800),
        })
        .await
        .unwrap();

    assert!(presigned
        .upload_url
        .contains("X-Amz-Algorithm=AWS4-HMAC-SHA256"));
    assert!(presigned
        .upload_url
        .contains("X-Amz-Credential=AKIAIOSFODNN7EXAMPLE"));
    assert_eq!(
        presigned.headers.get("Content-Type").unwrap(),
        "application/pdf"
    );
    assert!(presigned
        .file_url
        .starts_with("https://cdn.sthenocommerce.com/uploads/"));

    // Delete
    service.delete_file(&file.id).await.unwrap();
    let deleted_res = service.get_file(&file.id).await;
    assert!(deleted_res.is_err());
}
