use std::collections::BTreeMap;

use serde_json::json;
use stheno_payment::dto::{
    validate_credential_map, CredentialField, CredentialKind, CredentialMap,
};

const STRIPE_SCHEMA: &[CredentialField] = &[
    CredentialField {
        key: "api_key",
        label: "API key",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "webhook_secret",
        label: "Webhook secret",
        kind: CredentialKind::Secret,
        required: true,
    },
    CredentialField {
        key: "return_url",
        label: "Return URL",
        kind: CredentialKind::Url,
        required: false,
    },
];

const PEM_SCHEMA: &[CredentialField] = &[CredentialField {
    key: "private_key",
    label: "Private key",
    kind: CredentialKind::Pem,
    required: true,
}];

fn credentials(entries: &[(&str, serde_json::Value)]) -> CredentialMap {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.clone()))
        .collect::<BTreeMap<_, _>>()
}

#[test]
fn validates_required_secret_fields() {
    let err = validate_credential_map(
        STRIPE_SCHEMA,
        &credentials(&[("api_key", json!("sk_test_123"))]),
    )
    .expect_err("missing required webhook_secret must fail");

    match err {
        stheno_types::Error::Invalid { code, field, .. } => {
            assert_eq!(code, "credential_field_required");
            assert_eq!(field.as_deref(), Some("webhook_secret"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_invalid_urls_at_write_time() {
    let err = validate_credential_map(
        STRIPE_SCHEMA,
        &credentials(&[
            ("api_key", json!("sk_test_123")),
            ("webhook_secret", json!("whsec_123")),
            ("return_url", json!("not a url")),
        ]),
    )
    .expect_err("invalid URL must fail before use");

    match err {
        stheno_types::Error::Invalid { code, field, .. } => {
            assert_eq!(code, "credential_url_invalid");
            assert_eq!(field.as_deref(), Some("return_url"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn rejects_invalid_pem_at_write_time() {
    let err = validate_credential_map(
        PEM_SCHEMA,
        &credentials(&[(
            "private_key",
            json!("-----BEGIN KEY-----\nnot valid\n-----END KEY-----"),
        )]),
    )
    .expect_err("invalid PEM must fail before use");

    match err {
        stheno_types::Error::Invalid { code, field, .. } => {
            assert_eq!(code, "credential_pem_invalid");
            assert_eq!(field.as_deref(), Some("private_key"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn accepts_valid_url_pem_bool_and_secret_shapes() {
    const SCHEMA: &[CredentialField] = &[
        CredentialField {
            key: "api_key",
            label: "API key",
            kind: CredentialKind::Secret,
            required: true,
        },
        CredentialField {
            key: "public_key",
            label: "Public key",
            kind: CredentialKind::Pem,
            required: true,
        },
        CredentialField {
            key: "callback_url",
            label: "Callback URL",
            kind: CredentialKind::Url,
            required: true,
        },
        CredentialField {
            key: "sandbox",
            label: "Sandbox",
            kind: CredentialKind::Bool,
            required: true,
        },
    ];

    let creds = credentials(&[
        ("api_key", json!("sk_test_123")),
        (
            "public_key",
            json!("-----BEGIN PUBLIC KEY-----\nQUJDRA==\n-----END PUBLIC KEY-----"),
        ),
        (
            "callback_url",
            json!("https://example.com/hooks/payment/slug"),
        ),
        ("sandbox", json!(true)),
    ]);

    validate_credential_map(SCHEMA, &creds).expect("valid credential map");
}
