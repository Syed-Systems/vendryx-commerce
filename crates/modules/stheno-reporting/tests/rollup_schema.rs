use diesel_async::RunQueryDsl;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[derive(diesel::QueryableByName)]
struct TableCheck {
    #[diesel(sql_type = diesel::sql_types::Text)]
    table_name: String,
}

#[tokio::test]
async fn test_reporting_daily_rollup_tables_exist() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let mut conn = pool.get().await.expect("conn");

    let tables: Vec<TableCheck> = diesel::sql_query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'reporting' ORDER BY table_name;"
    )
    .load(&mut conn)
    .await
    .expect("query tables");

    let table_names: Vec<String> = tables.into_iter().map(|t| t.table_name).collect();
    assert!(table_names.contains(&"daily_rollup".to_string()));
    assert!(table_names.contains(&"daily_product_rollup".to_string()));

    db.drop().await;
}
