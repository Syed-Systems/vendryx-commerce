use stheno_file::FileProvider;
use stheno_file_s3::{S3Config, S3FileProvider};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(endpoint: String) -> S3Config {
    S3Config {
        bucket: "public-media".to_string(),
        private_bucket: Some("private-media".to_string()),
        region: "auto".to_string(),
        endpoint: Some(endpoint),
        access_key_id: "test-access-key".to_string(),
        secret_access_key: "test-secret-key".to_string(),
        cdn_url: Some("https://media.example.test".to_string()),
    }
}

#[tokio::test]
async fn custom_endpoint_uses_auto_region_and_separate_buckets() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/public-media/uploads/public_01"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/private-media/private/private_01"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let provider = S3FileProvider::new(config(server.uri()));
    let public = provider
        .upload("public_01", "hero.webp", "image/webp", b"public", false)
        .await
        .expect("public upload succeeds");
    assert_eq!(public.url, "https://media.example.test/uploads/public_01");

    let private = provider
        .upload(
            "private_01",
            "proof.pdf",
            "application/pdf",
            b"private",
            true,
        )
        .await
        .expect("private upload succeeds");
    assert_eq!(private.url, "s3://private-media/private/private_01");
    assert!(!private.url.starts_with("https://"));

    let requests = server.received_requests().await.expect("requests captured");
    assert!(requests.iter().all(|request| {
        request
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|authorization| authorization.contains("/auto/s3/aws4_request"))
    }));
}

#[tokio::test]
async fn private_presigned_upload_uses_private_bucket_without_public_file_url() {
    let server = MockServer::start().await;
    let provider = S3FileProvider::new(config(server.uri()));
    let result = provider
        .get_presigned_upload_url(
            "private_02",
            "payment-proof.pdf",
            "application/pdf",
            900,
            true,
        )
        .await
        .expect("private upload URL is signed");

    assert!(result
        .upload_url
        .contains("/private-media/private/private_02?"));
    assert!(result.upload_url.contains("auto%2Fs3%2Faws4_request"));
    assert!(result.upload_url.contains("X-Amz-Signature="));
    assert_eq!(result.file_url, "s3://private-media/private/private_02");
}

#[tokio::test]
async fn resolving_a_private_object_never_returns_a_public_url() {
    let server = MockServer::start().await;
    Mock::given(method("HEAD"))
        .and(path("/private-media/private/private_03"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let provider = S3FileProvider::new(config(server.uri()));
    let url = provider
        .get_url("private_03")
        .await
        .expect("private storage URI is returned");
    assert_eq!(url, "s3://private-media/private/private_03");
    assert!(!url.starts_with("https://"));
}
