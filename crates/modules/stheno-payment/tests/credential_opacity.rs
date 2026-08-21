use chrono::Utc;
use serde_json::to_string;
use stheno_payment::dto::{
    CredentialFieldReadDTO, CredentialHealth, CredentialKind, PaymentProviderCredentialDTO,
};

#[test]
fn serialized_credential_read_dto_contains_no_secret_material() {
    let api_key = "sk_live_secret_that_must_not_render";
    let webhook_secret = "whsec_secret_that_must_not_render";

    let dto = PaymentProviderCredentialDTO {
        id: "ppcred_1".to_string(),
        provider_id: "stripe".to_string(),
        credential_version: 7,
        webhook_slug: "hook_opaque_slug".to_string(),
        health: CredentialHealth::Configured,
        fields: vec![
            CredentialFieldReadDTO {
                key: "api_key".to_string(),
                label: "API key".to_string(),
                kind: CredentialKind::Secret,
                required: true,
                configured: true,
            },
            CredentialFieldReadDTO {
                key: "webhook_secret".to_string(),
                label: "Webhook secret".to_string(),
                kind: CredentialKind::Secret,
                required: true,
                configured: true,
            },
        ],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let serialized = to_string(&dto).expect("serialize credential DTO");

    assert!(!serialized.contains(api_key));
    assert!(!serialized.contains(webhook_secret));
    assert!(!serialized.contains("ciphertext"));
    assert!(!serialized.contains("nonce"));
}
