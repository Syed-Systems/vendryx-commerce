use serde_json::json;
use std::sync::Arc;
use stheno_links::LinkStore;
use stheno_query::QueryEngine;

#[test]
fn test_batched_join_asserts_zero_n_plus_one_queries() {
    let link_store = Arc::new(LinkStore::new());
    // Link 100 products to sales channels
    for i in 0..100 {
        link_store.link("product_sales_channel", format!("prod_{i}"), "sc_web");
    }

    let query_engine = QueryEngine::new(link_store);

    let mut products: Vec<serde_json::Value> = (0..100)
        .map(|i| json!({ "id": format!("prod_{i}"), "title": format!("Product {i}") }))
        .collect();

    query_engine.reset_query_count();

    // Execute batched join for 100 records
    query_engine.execute_batched_join(
        "product_sales_channel",
        &mut products,
        "id",
        "sales_channels",
    );

    // Verified: Exactly ONE batch query executed for all 100 records (Zero N+1!)
    assert_eq!(query_engine.query_count(), 1);
    assert_eq!(products[0]["sales_channels"][0], "sc_web");
    assert_eq!(products[99]["sales_channels"][0], "sc_web");
}
