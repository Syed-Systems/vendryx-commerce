//! Checkout state outlives the process that wrote it.
//!
//! `CompleteCartWorkflowEngine` kept its idempotency cache, its resume checkpoints and
//! its cart->order map in three plain `HashMap`s. A restart emptied all three, so a
//! customer's retry after a deploy produced a *second* order. Two replicas were worse:
//! neither ever saw the other's map.
//!
//! The in-memory `in_flight` mutex is still per-process, and deliberately so — the
//! cross-replica exclusion is the durable lease, and this test's second engine stands in
//! for a second replica.

use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_cart::{CartService, CreateAddressInput, CreateCartInput, CreateLineItemInput};
use stheno_flows::{CompleteCartInput, CompleteCartWorkflowEngine, StepFailureMode};
use stheno_order::OrderService;
use stheno_payment::PaymentService;
use stheno_promotion::PromotionService;
use stheno_workflow::WorkflowStore;

fn engine(pool: &stheno_db::DbPool) -> CompleteCartWorkflowEngine {
    CompleteCartWorkflowEngine::new(
        Arc::new(CartService::new(pool.clone())),
        Arc::new(OrderService::new(pool.clone())),
        Arc::new(PaymentService::new(pool.clone())),
        Arc::new(PromotionService::new(pool.clone())),
        Arc::new(stheno_links::PgLinkStore::new(pool.clone())),
        Arc::new(WorkflowStore::from_pool(pool.clone())),
    )
}

async fn seed(pool: &stheno_db::DbPool) -> stheno_cart::CartDTO {
    CartService::new(pool.clone())
        .create_cart(CreateCartInput {
            region_id: Some("reg_us".to_string()),
            customer_id: Some("cus_durable".to_string()),
            sales_channel_id: None,
            email: Some("durable@example.com".to_string()),
            currency_code: "usd".to_string(),
            shipping_address: Some(CreateAddressInput {
                first_name: Some("Dur".to_string()),
                last_name: Some("Able".to_string()),
                address_1: Some("1 Durable Way".to_string()),
                city: Some("New York".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("10001".to_string()),
                ..Default::default()
            }),
            billing_address: None,
            items: Some(vec![CreateLineItemInput {
                title: "Sturdy Chair".to_string(),
                subtitle: None,
                thumbnail: None,
                quantity: 1,
                unit_price: dec!(120.00),
                is_tax_inclusive: Some(false),
                compare_at_unit_price: None,
                requires_shipping: Some(true),
                is_discountable: Some(true),
                is_custom_price: Some(false),
                variant_id: None,
                product_id: None,
                product_title: None,
                product_description: None,
                product_subtitle: None,
                product_type: None,
                product_collection: None,
                product_handle: None,
                variant_sku: None,
                variant_barcode: None,
                variant_title: None,
                variant_option_values: None,
                metadata: None,
            }]),
            metadata: None,
        })
        .await
        .expect("cart created")
}

#[tokio::test]
async fn a_second_engine_does_not_re_complete_the_same_checkout() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let cart = seed(&pool).await;
    let idem = "durable_checkout_key".to_string();

    // Engine A: a fresh process that knows nothing.
    let first = engine(&pool)
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some(idem.clone()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("first complete succeeded");

    // Engine B: what a restart or a second replica looks like. Same database, no shared
    // memory. Under the old HashMap this re-ran the whole workflow and made a new order.
    let second = engine(&pool)
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some(idem.clone()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("second complete succeeded");

    assert_eq!(
        first.order.id, second.order.id,
        "a second engine must return the order the first one created, not a new one"
    );
    assert_eq!(
        second.order.summary.unwrap().paid_total,
        dec!(120.00),
        "the customer must be charged once, not twice"
    );

    db.drop().await;
}

#[tokio::test]
async fn the_idempotency_record_is_actually_in_the_database() {
    let db = stheno_db::testing::TestDb::migrated().await;
    let pool = stheno_db::create_pool(&db.url, 5).expect("pool");
    let cart = seed(&pool).await;
    let idem = "durable_row_check".to_string();

    engine(&pool)
        .execute(
            CompleteCartInput {
                cart_id: cart.id.clone(),
                idempotency_key: Some(idem.clone()),
            },
            StepFailureMode::None,
        )
        .await
        .expect("complete succeeded");

    // Proves the claim above is real rather than an artifact of a still-warm cache: the
    // row is counted directly, with no engine in the picture.
    let rows = db
        .query_text("SELECT count(*) FROM workflow.execution WHERE id = 'durable_row_check'")
        .await;

    assert_eq!(
        rows.trim(),
        "1",
        "the completed execution must be a row in workflow.execution, not process memory"
    );

    db.drop().await;
}
