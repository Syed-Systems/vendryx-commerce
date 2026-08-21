//! P31-06: Per-tenant data export (JSON bundle) before purge for deletion requests.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

#[tokio::test]
async fn tenant_export_produces_complete_bundle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    // 1. Create a tenant with domain
    service
        .create_tenant(NewTenant {
            id: "ten_export_me".to_string(),
            name: "Exportable Merchant".to_string(),
            tier: "pro".to_string(),
            domain: Some("export.localhost".to_string()),
        })
        .await
        .expect("tenant created");

    // 2. Export tenant data
    let bundle = service
        .export_tenant_data("ten_export_me")
        .await
        .expect("export bundle generated");

    assert_eq!(bundle.tenant.id, "ten_export_me");
    assert_eq!(bundle.tenant.name, "Exportable Merchant");
    assert!(bundle.subscription.is_some());
    assert_eq!(bundle.subscription.unwrap().tier, "pro");
    assert_eq!(bundle.domains.len(), 1);
    assert_eq!(bundle.domains[0].domain, "export.localhost");
}
