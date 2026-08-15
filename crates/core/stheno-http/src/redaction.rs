//! `stheno-http::redaction`: PII, token, and secret redaction layer (SECURITY §11.1, AGENTS.md §10).

use serde_json::Value;

const SENSITIVE_KEYS: &[&str] = &[
    "password",
    "token",
    "secret",
    "card_number",
    "cvv",
    "api_key",
    "authorization",
    "access_token",
    "refresh_token",
];

pub fn redact_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower_key = k.to_lowercase();
                if SENSITIVE_KEYS.iter().any(|&s| lower_key.contains(s)) {
                    *v = Value::String("[REDACTED]".to_string());
                } else {
                    redact_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                redact_value(item);
            }
        }
        _ => {}
    }
}
