//! P9-32: social draft creation calls the order workflow directly — no email, no
//! customer_id, buyer in metadata, sales_channel_id set at creation, source = "social".
//! (PRODUCT.md 5.5, traps 1-3.)

use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows_order::{
    should_skip_placed_push, SocialDraftInput, SocialDraftWorkflow, SOCIAL_SOURCE,
};
use stheno_order::{CreateLineItemInput, OrderService};

async fn workflow(db: &TestDb) -> SocialDraftWorkflow {
    let pool = create_pool(&db.url, 5).expect("pool");
    SocialDraftWorkflow::new(Arc::new(OrderService::new(pool)))
}

fn item() -> CreateLineItemInput {
    CreateLineItemInput {
        title: "Stheno Signature Tee".to_string(),
        quantity: 1,
        unit_price: rust_decimal::Decimal::new(2500, 2),
        ..Default::default()
    }
}

fn valid_input() -> SocialDraftInput {
    SocialDraftInput {
        sales_channel_id: Some("sc_whatsapp".to_string()),
        region_id: Some("reg_us".to_string()),
        currency_code: "usd".to_string(),
        items: vec![item()],
        buyer_name: Some("Amina".to_string()),
        buyer_phone: Some("+15551234567".to_string()),
        source_ref: Some("ig_story_42".to_string()),
    }
}

#[tokio::test]
async fn trap_1_a_social_draft_carries_no_email_and_no_customer_id() {
    let db = TestDb::migrated().await;
    let order = workflow(&db)
        .await
        .create_social_draft(valid_input())
        .await
        .unwrap();

    assert_eq!(
        order.email, None,
        "an email would silently create a guest customer reused for every later order"
    );
    assert_eq!(order.customer_id, None, "buyers never have accounts");
    assert!(order.is_draft_order);
    db.drop().await;
}

#[tokio::test]
async fn trap_1_buyer_identity_lives_in_metadata() {
    let db = TestDb::migrated().await;
    let order = workflow(&db)
        .await
        .create_social_draft(valid_input())
        .await
        .unwrap();
    let metadata = order.metadata.expect("metadata must be set");

    assert_eq!(metadata["buyer_name"], "Amina");
    assert_eq!(metadata["buyer_phone"], "+15551234567");
    assert_eq!(
        metadata["ref"], "ig_story_42",
        "?ref= attribution carries through"
    );
    db.drop().await;
}

#[tokio::test]
async fn trap_2_creation_is_rejected_without_a_sales_channel() {
    let db = TestDb::migrated().await;
    let mut input = valid_input();
    input.sales_channel_id = None;

    let err = workflow(&db)
        .await
        .create_social_draft(input)
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("sales_channel_id"),
        "error must name the missing field, got: {err}"
    );
    db.drop().await;
}

#[tokio::test]
async fn trap_2_a_blank_sales_channel_is_treated_as_missing() {
    let db = TestDb::migrated().await;
    let mut input = valid_input();
    input.sales_channel_id = Some("   ".to_string());

    assert!(
        workflow(&db)
            .await
            .create_social_draft(input)
            .await
            .is_err(),
        "whitespace must not satisfy the requirement"
    );
    db.drop().await;
}

#[tokio::test]
async fn trap_2_sales_channel_is_stamped_on_the_draft() {
    let db = TestDb::migrated().await;
    let order = workflow(&db)
        .await
        .create_social_draft(valid_input())
        .await
        .unwrap();
    assert_eq!(order.sales_channel_id.as_deref(), Some("sc_whatsapp"));
    db.drop().await;
}

#[tokio::test]
async fn trap_3_source_is_stamped_social_at_creation() {
    let db = TestDb::migrated().await;
    let order = workflow(&db)
        .await
        .create_social_draft(valid_input())
        .await
        .unwrap();
    let metadata = order.metadata.clone().unwrap();

    assert_eq!(metadata["source"], SOCIAL_SOURCE);
    assert!(
        should_skip_placed_push(&order),
        "the push subscriber must skip social orders"
    );
    db.drop().await;
}

#[tokio::test]
async fn trap_3_a_non_social_order_still_pushes() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let order = order_service
        .create_order(stheno_order::CreateOrderInput {
            currency_code: "usd".to_string(),
            items: Some(vec![item()]),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(
        !should_skip_placed_push(&order),
        "a normal checkout must still notify the merchant"
    );
    db.drop().await;
}

#[tokio::test]
async fn an_empty_basket_is_rejected() {
    let db = TestDb::migrated().await;
    let mut input = valid_input();
    input.items = vec![];
    assert!(workflow(&db)
        .await
        .create_social_draft(input)
        .await
        .is_err());
    db.drop().await;
}
