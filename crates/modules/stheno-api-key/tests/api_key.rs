//! Test P7-16, P14-05: API Key creation, validation, scoping, and revocation on Postgres.

use stheno_api_key::{ApiKeyService, CreateApiKeyInput, UpdateApiKeyInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_api_key_lifecycle_and_scope_validation() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = ApiKeyService::new(pool);

    // 1. Create publishable API key scoped to specific sales channels (F20)
    let created = service
        .create_api_key(CreateApiKeyInput {
            key_type: "publishable".to_string(),
            title: "Web Storefront Key".to_string(),
            sales_channel_ids: Some(vec!["sc_web_us".to_string(), "sc_web_eu".to_string()]),
            permissions: None,
            created_by: Some("user_admin".to_string()),
            metadata: None,
        })
        .await
        .unwrap();

    assert!(created.token.starts_with("pk_"));
    assert!(created.api_key.redacted_key.starts_with("pk_..."));
    assert_eq!(created.api_key.sales_channel_ids.len(), 2);
    assert!(!created.api_key.is_revoked);

    // 2. Validate key
    let valid_res = service.validate_key(&created.token).await.unwrap();
    assert!(valid_res.is_valid);
    assert_eq!(valid_res.sales_channel_ids, vec!["sc_web_us", "sc_web_eu"]);

    // 3. Update scope
    let updated = service
        .update_key(
            &created.api_key.id,
            UpdateApiKeyInput {
                title: Some("Global Web Storefront Key".to_string()),
                sales_channel_ids: Some(vec![
                    "sc_web_us".to_string(),
                    "sc_web_eu".to_string(),
                    "sc_web_asia".to_string(),
                ]),
                permissions: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.sales_channel_ids.len(), 3);

    // 4. Revoke key
    let revoked = service
        .revoke_key(&created.api_key.id, Some("user_admin"))
        .await
        .unwrap();
    assert!(revoked.is_revoked);

    // 5. Validation of revoked key fails
    let post_revoke_res = service.validate_key(&created.token).await.unwrap();
    assert!(!post_revoke_res.is_valid);

    db.drop().await;
}
