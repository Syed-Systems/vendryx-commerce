use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_file::{FileService, UploadFileInput, UploadPipeline};
use stheno_file_local::LocalFileProvider;

#[tokio::test]
async fn test_magic_byte_sniffing_and_svg_rejection() {
    // Valid PNG header: 89 50 4E 47 0D 0A 1A 0A
    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
    let detected = UploadPipeline::sniff_mime_type(&png_bytes).unwrap();
    assert_eq!(detected.as_str(), "image/png");

    // Valid JPEG header: FF D8 FF E0
    let jpeg_bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
    let detected_jpeg = UploadPipeline::sniff_mime_type(&jpeg_bytes).unwrap();
    assert_eq!(detected_jpeg.as_str(), "image/jpeg");

    // SVG must be unconditionally rejected
    let svg_bytes = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    let svg_res = UploadPipeline::sniff_mime_type(svg_bytes);
    assert!(svg_res.is_err());
    match svg_res.unwrap_err() {
        stheno_types::Error::Invalid { code, .. } => assert_eq!(code, "svg_disallowed"),
        other => panic!("Expected Error::Invalid, got {:?}", other),
    }
}

#[tokio::test]
async fn test_local_file_provider_upload_and_presigned() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let provider = Arc::new(LocalFileProvider::new(
        "./uploads",
        "https://cdn.example.com",
    ));
    let service = FileService::new(provider, pool);

    // Upload PNG file
    let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x01, 0x02];
    let file = service
        .upload_file(UploadFileInput {
            filename: "avatar.png".to_string(),
            content_type: Some("image/png".to_string()),
            bytes: png_bytes,
            metadata: None,
        })
        .await
        .unwrap();

    assert_eq!(file.filename, "avatar.png");
    assert_eq!(file.mime_type, "image/png");
    assert!(file.url.starts_with("https://cdn.example.com/uploads/"));
    assert_eq!(file.size_bytes, 10);

    // Get file
    let fetched = service.get_file(&file.id).await.unwrap();
    assert_eq!(fetched.id, file.id);

    // Presigned upload URL
    let presigned = service
        .create_presigned_upload(stheno_file::CreatePresignedUploadInput {
            filename: "document.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            expires_in_seconds: Some(3600),
        })
        .await
        .unwrap();

    assert!(presigned.upload_url.contains("/api/upload/direct/"));
    assert!(presigned.file_url.ends_with(".pdf"));

    // Delete file
    service.delete_file(&file.id).await.unwrap();
    let deleted_res = service.get_file(&file.id).await;
    assert!(deleted_res.is_err());

    db.drop().await;
}
