use std::sync::Arc;

use rust_decimal_macros::dec;
use stheno_db::create_pool;
use stheno_db::tenant;
use stheno_db::testing::TestDb;
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};

fn order_input(product_id: &str, product_type: &str, draft: bool) -> CreateOrderInput {
    CreateOrderInput {
        currency_code: "usd".to_string(),
        is_draft_order: Some(draft),
        items: Some(vec![CreateLineItemInput {
            title: "Sample product".to_string(),
            quantity: 2,
            unit_price: dec!(10.00),
            product_id: Some(product_id.to_string()),
            product_title: Some("Sample product".to_string()),
            product_type: Some(product_type.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    }
}

#[tokio::test]
async fn aggregates_only_current_tenant_non_draft_orders() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = Arc::new(OrderService::new(pool));

    tenant::scope("ten_store_alpha", async {
        service
            .create_order(order_input("prod_alpha", "Accessories", false))
            .await
            .expect("alpha order");
        service
            .create_order(order_input("prod_alpha_draft", "Accessories", true))
            .await
            .expect("alpha draft order");
    })
    .await;

    tenant::scope("ten_store_beta", async {
        service
            .create_order(order_input("prod_beta", "Accessories", false))
            .await
            .expect("beta order");
    })
    .await;

    let as_of = chrono::Utc::now();
    let current_start = as_of - chrono::Duration::days(7);
    let comparison_start = current_start - chrono::Duration::days(7);
    let products = tenant::scope("ten_store_alpha", async {
        service
            .store_intelligence_order_signals(current_start, comparison_start, as_of)
            .await
    })
    .await
    .expect("alpha product signals");
    assert_eq!(products.len(), 1);
    assert_eq!(products[0].product_id, "prod_alpha");
    assert_eq!(products[0].current_units, 2);
    assert_eq!(products[0].current_orders, 1);

    let categories = tenant::scope("ten_store_alpha", async {
        service
            .store_intelligence_category_signals(current_start, comparison_start, as_of)
            .await
    })
    .await
    .expect("alpha category signals");
    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].category, "Accessories");
    assert_eq!(categories[0].current_units, 2);

    let beta_products = tenant::scope("ten_store_beta", async {
        service
            .store_intelligence_order_signals(current_start, comparison_start, as_of)
            .await
    })
    .await
    .expect("beta product signals");
    assert_eq!(beta_products.len(), 1);
    assert_eq!(beta_products[0].product_id, "prod_beta");

    db.drop().await;
}
