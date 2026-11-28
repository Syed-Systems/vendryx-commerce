//! P31-05: Retention window then hard purge.
//! A closed tenant's rows are deleted after N days after the caller's module cleanup step.

use std::future::Future;
use std::pin::Pin;

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

#[tokio::test]
async fn tenant_purge_removes_all_records_after_retention_window() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    // 1. Create a tenant
    service
        .create_tenant(NewTenant {
            id: "ten_purge_me".to_string(),
            name: "Purge Me Ltd".to_string(),
            tier: "free".to_string(),
            domain: Some("purgeme.localhost".to_string()),
        })
        .await
        .expect("tenant created");

    // 2. Verify tenant and domain exist
    assert!(service.get_tenant("ten_purge_me").await.is_ok());
    assert_eq!(
        service
            .tenant_for_domain("purgeme.localhost")
            .await
            .unwrap(),
        "ten_purge_me"
    );

    // 3. Close tenant
    service
        .set_tenant_status(
            "ten_purge_me",
            "closed",
            "Customer requested immediate purge",
        )
        .await
        .expect("tenant closed");

    // 4. Purge with retention_days = 0 (immediate)
    let result = service
        .purge_closed_tenants_with(0, |_| {
            Box::pin(async { Ok(()) })
                as Pin<Box<dyn Future<Output = stheno_types::Result<()>> + Send>>
        })
        .await
        .expect("purge succeeded");
    assert_eq!(result.purged_count, 1);
    assert_eq!(result.purged_tenant_ids, vec!["ten_purge_me"]);

    // 5. Assert tenant and domain are completely gone
    assert!(service.get_tenant("ten_purge_me").await.is_err());
    assert!(service
        .tenant_for_domain("purgeme.localhost")
        .await
        .is_err());
}
