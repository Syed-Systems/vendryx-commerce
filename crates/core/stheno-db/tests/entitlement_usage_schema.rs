use stheno_db::create_pool;
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn test_entitlement_usage_schema_exists() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let mut conn = pool.get().await.expect("conn");

    let rows: Vec<stheno_db::UsageRow> =
        diesel::sql_query("SELECT used FROM entitlement_usage WHERE resource = 'products';")
            .load(&mut conn)
            .await
            .expect("query");

    assert!(
        !rows.is_empty(),
        "entitlement_usage table must be seeded with products"
    );
    assert_eq!(rows[0].used, 0);

    db.drop().await;
}
