use serde_json::json;
use stheno_notification::NotificationProvider;
use stheno_notification_resend::{ResendConfig, ResendNotificationProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_resend_successful_email_dispatch() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Authorization", "Bearer test_resend_api_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "re_abcdef123456",
            "from": "Vendryx <notifications@vendryx.store>",
            "to": ["merchant@example.com"],
            "created_at": "2026-09-09T00:00:00.000Z"
        })))
        .mount(&mock_server)
        .await;

    let config = ResendConfig {
        api_key: "test_resend_api_key".to_string(),
        from_email: "notifications@vendryx.store".to_string(),
        from_name: Some("Vendryx".to_string()),
        base_url: Some(mock_server.uri()),
    };

    let provider = ResendNotificationProvider::new(config);

    let result = provider
        .send(
            "merchant@example.com",
            "email",
            Some("Welcome to Vendryx"),
            "<h1>Welcome to your new store</h1>",
            None,
        )
        .await
        .expect("send succeeds");

    assert_eq!(
        result.provider_message_id.as_deref(),
        Some("re_abcdef123456")
    );
    let meta = result.metadata.expect("metadata present");
    assert_eq!(meta["provider"], "resend");
    assert_eq!(meta["message_id"], "re_abcdef123456");

    let records = provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "merchant@example.com");
    assert_eq!(records[0].from, "Vendryx <notifications@vendryx.store>");
    assert_eq!(records[0].subject, "Welcome to Vendryx");
}

#[tokio::test]
async fn resend_uses_idempotency_header_for_retryable_delivery() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/emails"))
        .and(header("Idempotency-Key", "trial-warning/ten_acme/day-3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "re_trial_warning" })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let provider = ResendNotificationProvider::new(ResendConfig {
        api_key: "test_resend_api_key".to_string(),
        from_email: "notifications@vendryx.store".to_string(),
        from_name: None,
        base_url: Some(mock_server.uri()),
    });
    provider
        .send_idempotent(
            "merchant@example.com",
            "email",
            Some("Trial reminder"),
            "Your trial ends soon.",
            None,
            Some("trial-warning/ten_acme/day-3"),
        )
        .await
        .expect("provider accepts the stable key");
    mock_server.verify().await;
}

#[tokio::test]
async fn test_resend_error_response_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "statusCode": 422,
            "message": "Domain not verified",
            "name": "validation_error"
        })))
        .mount(&mock_server)
        .await;

    let config = ResendConfig {
        api_key: "test_key".to_string(),
        from_email: "unverified@domain.com".to_string(),
        from_name: None,
        base_url: Some(mock_server.uri()),
    };

    let provider = ResendNotificationProvider::new(config);

    let err = provider
        .send("user@example.com", "email", Some("Subject"), "Body", None)
        .await
        .expect_err("should fail with 422");

    match err {
        stheno_types::Error::ProviderFailed { provider, source } => {
            assert_eq!(provider, "resend");
            assert!(source.to_string().contains("422"));
            assert!(source.to_string().contains("Domain not verified"));
        }
        other => panic!("expected ProviderFailed, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_resend_unsupported_channel_rejected() {
    let config = ResendConfig {
        api_key: "key".to_string(),
        from_email: "test@vendryx.store".to_string(),
        from_name: None,
        base_url: None,
    };

    let provider = ResendNotificationProvider::new(config);

    let err = provider
        .send("1234567890", "sms", Some("SMS"), "Text", None)
        .await
        .expect_err("should reject sms");

    match err {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "unsupported_channel");
        }
        other => panic!("expected Invalid, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_resend_invalid_recipient_rejected() {
    let config = ResendConfig {
        api_key: "key".to_string(),
        from_email: "test@vendryx.store".to_string(),
        from_name: None,
        base_url: None,
    };

    let provider = ResendNotificationProvider::new(config);

    let err = provider
        .send("invalid-email", "email", Some("Subject"), "Text", None)
        .await
        .expect_err("should reject invalid email");

    match err {
        stheno_types::Error::Invalid { code, .. } => {
            assert_eq!(code, "invalid_email_recipient");
        }
        other => panic!("expected Invalid, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_resend_development_mock_mode() {
    let config = ResendConfig {
        api_key: "".to_string(),
        from_email: "noreply@vendryx.store".to_string(),
        from_name: Some("Vendryx Store".to_string()),
        base_url: None,
    };

    let provider = ResendNotificationProvider::new(config);

    let result = provider
        .send(
            "dev@example.com",
            "email",
            Some("Development Reset Token"),
            "Token: 123456",
            None,
        )
        .await
        .expect("local dev mode succeeds without api key");

    assert!(result.provider_message_id.unwrap().starts_with("re_mock_"));
    let records = provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "dev@example.com");
}
