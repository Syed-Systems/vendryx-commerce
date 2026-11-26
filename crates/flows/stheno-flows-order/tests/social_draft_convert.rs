//! P9-33: Trap-2 regression — create a social draft, then convert it. Reservation
//! location resolution reads `sales_channel_id` off the order, so conversion is the
//! moment a draft missing it would explode. (PRODUCT.md 5.5, trap 2.)

use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_flows_order::{SocialDraftInput, SocialDraftWorkflow, should_skip_placed_push};
use stheno_order::{CreateLineItemInput, CreateOrderInput, OrderService};

fn item() -> CreateLineItemInput {
    CreateLineItemInput {
        title: "Stheno Performance Hoodie".to_string(),
        quantity: 2,
        unit_price: rust_decimal::Decimal::new(7900, 2),
        ..Default::default()
    }
}

fn input() -> SocialDraftInput {
    SocialDraftInput {
        sales_channel_id: Some("sc_whatsapp".to_string()),
        region_id: Some("reg_us".to_string()),
        currency_code: "usd".to_string(),
        items: vec![item()],
        buyer_name: Some("Kwame".to_string()),
        buyer_phone: Some("+15550001111".to_string()),
        delivery_address: None,
        shipping_method: None,
        payment_preference: None,
        source_event_id: None,
        source_ref: None,
    }
}

#[tokio::test]
async fn a_social_draft_converts_and_keeps_its_sales_channel() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let workflow = SocialDraftWorkflow::new(order_service.clone());

    let draft = workflow.create_social_draft(input()).await.unwrap();
    assert!(draft.is_draft_order);

    let converted = workflow.convert_to_order(&draft.id).await.unwrap();

    // This is the assertion the trap is about: the channel must survive conversion,
    // because reservation follows it to the stock location.
    assert_eq!(
        converted.sales_channel_id.as_deref(),
        Some("sc_whatsapp"),
        "reservation cannot resolve a location without this"
    );
    assert_eq!(converted.id, draft.id);
    assert_eq!(converted.status, "pending");
    let (queued, count) = order_service
        .list_social_drafts(20, 0)
        .await
        .expect("list unconfirmed social drafts");
    assert_eq!(count, 0, "confirmed order leaves the review queue");
    assert!(queued.is_empty());

    db.drop().await;
}

#[tokio::test]
async fn confirming_the_same_social_draft_twice_returns_the_existing_order() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let workflow = SocialDraftWorkflow::new(Arc::new(OrderService::new(pool)));

    let draft = workflow.create_social_draft(input()).await.unwrap();
    let converted = workflow.convert_to_order(&draft.id).await.unwrap();
    assert_eq!(converted.status, "pending");
    let retry = workflow
        .convert_to_order(&draft.id)
        .await
        .expect("replayed confirmation returns the existing social order");
    assert_eq!(retry.id, converted.id);
    assert_eq!(retry.status, "pending");
    assert!(!retry.is_draft_order);

    db.drop().await;
}

#[tokio::test]
async fn conversion_preserves_the_social_marker_so_push_stays_suppressed() {
    // Conversion fires order.placed exactly like a real checkout. Without the marker
    // surviving, the merchant gets notified about an order their own staff just accepted.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let workflow = SocialDraftWorkflow::new(Arc::new(OrderService::new(pool)));

    let draft = workflow.create_social_draft(input()).await.unwrap();
    let converted = workflow.convert_to_order(&draft.id).await.unwrap();

    assert!(
        should_skip_placed_push(&converted),
        "the social marker must survive conversion"
    );

    db.drop().await;
}

#[tokio::test]
async fn conversion_preserves_buyer_metadata() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let workflow = SocialDraftWorkflow::new(Arc::new(OrderService::new(pool)));

    let draft = workflow.create_social_draft(input()).await.unwrap();
    let converted = workflow.convert_to_order(&draft.id).await.unwrap();

    let metadata = converted
        .metadata
        .expect("metadata must survive conversion");
    assert_eq!(metadata["buyer_name"], "Kwame");
    assert_eq!(metadata["buyer_phone"], "+15550001111");
    assert_eq!(
        converted.email, None,
        "conversion must not invent a customer"
    );
    assert_eq!(converted.customer_id, None);

    db.drop().await;
}

#[tokio::test]
async fn converting_a_draft_without_a_sales_channel_is_rejected() {
    // Guard the path that bypasses create_social_draft: a draft created some other way,
    // missing the channel, must still be refused rather than failing inside reservation.
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let orphan = order_service
        .create_order(CreateOrderInput {
            currency_code: "usd".to_string(),
            items: Some(vec![item()]),
            is_draft_order: Some(true),
            sales_channel_id: None,
            ..Default::default()
        })
        .await
        .unwrap();

    let workflow = SocialDraftWorkflow::new(order_service);
    let err = workflow.convert_to_order(&orphan.id).await.unwrap_err();
    assert!(
        err.to_string().contains("sales_channel_id"),
        "error must name the cause, got: {err}"
    );

    db.drop().await;
}

#[tokio::test]
async fn converting_a_non_draft_order_is_rejected() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let order_service = Arc::new(OrderService::new(pool));
    let live = order_service
        .create_order(CreateOrderInput {
            currency_code: "usd".to_string(),
            items: Some(vec![item()]),
            sales_channel_id: Some("sc_web".to_string()),
            is_draft_order: Some(false),
            ..Default::default()
        })
        .await
        .unwrap();

    let workflow = SocialDraftWorkflow::new(order_service);
    assert!(workflow.convert_to_order(&live.id).await.is_err());

    db.drop().await;
}
