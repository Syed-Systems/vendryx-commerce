//! P33-01: Cross-tenant aggregation helper: one reviewed place where query crosses tenant boundary.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{NewTenant, PlatformService};

#[tokio::test]
async fn cross_tenant_aggregate_computes_platform_totals() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    let initial = service
        .get_cross_tenant_aggregate()
        .await
        .expect("initial query")
        .total_merchants;

    // Create 2 test tenants
    service
        .create_tenant(NewTenant {
            id: "ten_merch_a".to_string(),
            name: "Merchant A".to_string(),
            tier: "free".to_string(),
            domain: None,
        })
        .await
        .expect("tenant A created");

    service
        .create_tenant(NewTenant {
            id: "ten_merch_b".to_string(),
            name: "Merchant B".to_string(),
            tier: "pro".to_string(),
            domain: None,
        })
        .await
        .expect("tenant B created");

    let aggregate = service
        .get_cross_tenant_aggregate()
        .await
        .expect("cross tenant aggregate succeeds");

    assert_eq!(aggregate.total_merchants, initial + 2);
    assert_eq!(aggregate.active_merchants, initial + 2);
}
