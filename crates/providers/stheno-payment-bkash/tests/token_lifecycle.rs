//! P22-02: bKash grant-token lifecycle with refresh and expiry handling.

use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_payment_bkash::BkashPaymentProvider;

#[tokio::test]
async fn bkash_token_lifecycle_grants_and_caches_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .and(header("username", "test_username"))
        .and(header("password", "test_password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "statusMessage": "Successful",
            "id_token": "token_bkash_abc_123",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "refresh_token_xyz"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = BkashPaymentProvider::new(
        &mock_server.uri(),
        "test_app_key",
        "test_app_secret",
        "test_username",
        "test_password",
    );

    // Call 1: Grants token from remote server
    let token1 = provider.get_auth_token().await.expect("grant succeeds");
    assert_eq!(token1, "token_bkash_abc_123");

    // Call 2: Uses cached token without hitting mock server again
    let token2 = provider
        .get_auth_token()
        .await
        .expect("cached token reused");
    assert_eq!(token2, "token_bkash_abc_123");
}

#[tokio::test]
async fn bkash_token_lifecycle_handles_grant_failure() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "statusCode": "2001",
            "statusMessage": "Invalid App Key or App Secret"
        })))
        .mount(&mock_server)
        .await;

    let provider = BkashPaymentProvider::new(
        &mock_server.uri(),
        "bad_key",
        "bad_secret",
        "test_user",
        "test_pass",
    );

    let err = provider.get_auth_token().await.unwrap_err();
    assert!(matches!(err, stheno_types::Error::Invalid { .. }));
}
