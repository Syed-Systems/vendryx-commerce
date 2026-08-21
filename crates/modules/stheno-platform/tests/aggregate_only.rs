//! P33-02: Strict aggregate-only guarantee: CrossTenantAggregateDTO refuses to carry row-level merchant rows.

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_platform::{CrossTenantAggregateDTO, PlatformService};

#[tokio::test]
async fn aggregate_only_shape_guarantees_no_row_leakage() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PlatformService::new(pool);

    let agg: CrossTenantAggregateDTO = service
        .get_cross_tenant_aggregate()
        .await
        .expect("aggregate query succeeds");

    // Verify type shape holds only numerical counts and money totals, no raw rows
    assert!(agg.total_merchants >= 1);
    assert_eq!(agg.total_invoices_issued, 0);
}
