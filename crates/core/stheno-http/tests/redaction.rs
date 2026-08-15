use serde_json::json;
use stheno_http::redact_value;

#[test]
fn test_sensitive_field_redaction() {
    let mut payload = json!({
        "username": "alice",
        "password": "super_secret_password",
        "api_key": "sk_live_123456789",
        "card": {
            "card_number": "4111222233334444",
            "cvv": "123",
            "holder": "Alice Smith"
        }
    });

    redact_value(&mut payload);

    assert_eq!(payload["username"], "alice");
    assert_eq!(payload["password"], "[REDACTED]");
    assert_eq!(payload["api_key"], "[REDACTED]");
    assert_eq!(payload["card"]["card_number"], "[REDACTED]");
    assert_eq!(payload["card"]["cvv"], "[REDACTED]");
    assert_eq!(payload["card"]["holder"], "Alice Smith");
}
