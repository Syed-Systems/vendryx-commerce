use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant, QuotaCounter};

/// Counters decrement on delete and never go below zero.
///
/// Runs inside `with_tenant_conn` rather than a bare `pool.get()`: under ADR-012 a raw
/// pooled connection has no tenant scope, so `entitlement_usage.tenant_id` would take its
/// DEFAULT from an unset setting and violate NOT NULL. That failure is the intended one —
/// unscoped writes are supposed to be impossible, not silently attributed.
#[tokio::test]
async fn test_quota_decrement_on_delete() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());

    tenant::with_tenant_conn(&pool, move |conn| {
        Box::pin(async move {
            counter.set_usage(conn, "products", 2).await?;

            assert_eq!(counter.decrement(conn, "products").await?, 1);
            assert_eq!(counter.decrement(conn, "products").await?, 0);
            // Bounded by GREATEST(0, used - 1), so it stays at zero.
            assert_eq!(counter.decrement(conn, "products").await?, 0);

            Ok(())
        })
    })
    .await
    .expect("quota operations");

    db.drop().await;
}
