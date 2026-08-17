use stheno_db::testing::TestDb;
use stheno_db::{create_pool, QuotaCounter};

#[tokio::test]
async fn test_quota_decrement_on_delete() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let counter = QuotaCounter::new(pool.clone());
    let mut conn = pool.get().await.expect("conn");

    // Set usage to 2
    counter
        .set_usage(&mut conn, "products", 2)
        .await
        .expect("set");

    // Decrement once
    let d1 = counter
        .decrement(&mut conn, "products")
        .await
        .expect("dec 1");
    assert_eq!(d1, 1);

    // Decrement twice
    let d2 = counter
        .decrement(&mut conn, "products")
        .await
        .expect("dec 2");
    assert_eq!(d2, 0);

    // Decrement when at 0 stays at 0 (bounded by GREATEST(0, used - 1))
    let d3 = counter
        .decrement(&mut conn, "products")
        .await
        .expect("dec 3");
    assert_eq!(d3, 0);

    db.drop().await;
}
