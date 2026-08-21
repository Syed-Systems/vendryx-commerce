//! P31-08: A domain can belong to exactly one tenant, enforced by unique constraint.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

#[tokio::test]
async fn domain_uniqueness_prevents_double_claim() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    // 1. Create first tenant claiming "store.example.com"
    service
        .create_tenant(NewTenant {
            id: "ten_first".to_string(),
            name: "First Merchant".to_string(),
            tier: "free".to_string(),
            domain: Some("store.example.com".to_string()),
        })
        .await
        .expect("first tenant created");

    // 2. Create second tenant
    service
        .create_tenant(NewTenant {
            id: "ten_second".to_string(),
            name: "Second Merchant".to_string(),
            tier: "free".to_string(),
            domain: None,
        })
        .await
        .expect("second tenant created");

    // 3. Second tenant attempts to claim the same domain -> fails with Conflict
    let attempt = service
        .add_tenant_domain("ten_second", "store.example.com", false)
        .await;

    assert!(
        attempt.is_err(),
        "claiming an already-assigned domain must fail"
    );
    match attempt.unwrap_err() {
        stheno_types::Error::Conflict { code, .. } => {
            assert_eq!(code, "domain_already_claimed");
        }
        other => panic!("expected Conflict error, got {other:?}"),
    }
}
