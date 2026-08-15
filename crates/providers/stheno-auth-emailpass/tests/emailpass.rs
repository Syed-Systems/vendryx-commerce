//! Test P7-03: Email & Password Auth Provider.

use serde_json::json;
use std::sync::Arc;
use stheno_auth::{AuthProviderService, AuthService};
use stheno_auth_emailpass::EmailPassAuthProvider;

#[tokio::test]
async fn test_emailpass_provider_register_and_authenticate() {
    let auth_service = Arc::new(AuthService::new(None));
    let provider = EmailPassAuthProvider::new(auth_service);

    assert_eq!(provider.provider_id(), "emailpass");

    // 1. Register new user
    let reg_payload = json!({
        "email": "sarah@stheno.store",
        "password": "PasswordSarah123!",
        "actor_id": "cust_sarah",
        "actor_type": "customer"
    });

    let registered = provider.register(&reg_payload).await.unwrap();
    assert_eq!(registered.entity_id, "sarah@stheno.store");
    assert_eq!(registered.actor_id, Some("cust_sarah".to_string()));

    // 2. Authenticate
    let auth_payload = json!({
        "email": "sarah@stheno.store",
        "password": "PasswordSarah123!"
    });

    let authenticated = provider.authenticate(&auth_payload).await.unwrap();
    assert_eq!(authenticated.id, registered.id);

    // 3. Failed authentication with wrong password
    let bad_auth_payload = json!({
        "email": "sarah@stheno.store",
        "password": "WrongPassword"
    });

    assert!(provider.authenticate(&bad_auth_payload).await.is_err());
}
