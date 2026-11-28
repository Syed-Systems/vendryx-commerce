//! Index entries survive the service instance that wrote them.

use std::collections::HashMap;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_index::{IndexService, QueryIndexFilterInput, UpsertIndexDataInput};

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = IndexService::new(pool.clone());
    writer
        .upsert_index_data(UpsertIndexDataInput {
            entity_name: "product".to_string(),
            entity_id: "prod_outlive".to_string(),
            data: serde_json::json!({ "title": "Kept", "price": 10.0 }),
            version: Some(1),
        })
        .await
        .expect("upsert");
    drop(writer);

    let reader = IndexService::new(pool);
    let rows = reader
        .query_index(QueryIndexFilterInput {
            entity_name: "product".to_string(),
            filters: HashMap::new(),
            sort_by: None,
            sort_order: None,
            limit: Some(10),
            offset: Some(0),
        })
        .await
        .expect("query");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].entity_id, "prod_outlive");
    assert_eq!(rows[0].version, 1);
    assert_eq!(rows[0].data["title"], "Kept");

    db.drop().await;
}
