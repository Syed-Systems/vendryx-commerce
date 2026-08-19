use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant, QuotaCounter};

#[tokio::test]
async fn test_quota_atomic_conditional_increment() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    // A raw `pool.get()` has no tenant scope under ADR-012, so the counter's
    // `tenant_id` DEFAULT would resolve to NULL. Everything runs inside one scoped
    // transaction instead.
    tenant::with_tenant_conn(&pool, move |conn| {
        Box::pin(async move {
            // 1. Initial increment with limit of 3
            let u1 = counter
                .increment(conn, "products", Some(3))
                .await
                .expect("increment 1");
            assert_eq!(u1, 1);

            let u2 = counter
                .increment(conn, "products", Some(3))
                .await
                .expect("increment 2");
            assert_eq!(u2, 2);

            let u3 = counter
                .increment(conn, "products", Some(3))
                .await
                .expect("increment 3");
            assert_eq!(u3, 3);

            // 2. 4th increment must be rejected because limit is 3 (zero rows updated -> conflict)
            let res = counter.increment(conn, "products", Some(3)).await;
            assert!(res.is_err(), "4th create must be rejected at limit 3");

            // 3. Current count must still be 3
            let cur = counter.get_usage(conn, "products").await.expect("usage");
            assert_eq!(cur, 3);

            // 4. Unlimited increment (limit: None) succeeds
            let u4 = counter
                .increment(conn, "products", None)
                .await
                .expect("unlimited increment");
            assert_eq!(u4, 4);

            Ok(())
        })
    })
    .await
    .expect("quota operations");

    db.drop().await;
}
