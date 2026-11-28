use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use stheno_db::{create_pool, testing::TestDb};
use stheno_fulfillment::{
    CalculatePriceContext, CreateFulfillmentInput, CreateFulfillmentItemInput,
    CreateFulfillmentLabelInput, CreateFulfillmentResponse, FulfillmentOptionDTO,
    FulfillmentProvider, FulfillmentService, SocialDispatchResolution,
};
use stheno_types::{Error, Result};

struct SocialTestCourier {
    calls: AtomicUsize,
    ambiguous_after_accept: AtomicBool,
}

#[async_trait]
impl FulfillmentProvider for SocialTestCourier {
    fn identifier(&self) -> &'static str {
        "social_test_courier"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(Vec::new())
    }

    async fn validate_option(&self, _option_data: Option<&Value>) -> Result<bool> {
        Ok(true)
    }

    async fn calculate_price(
        &self,
        _option_data: Option<&Value>,
        _data: Option<&Value>,
        _context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        Ok(Decimal::ZERO)
    }

    async fn create_fulfillment(
        &self,
        data: Option<&Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&Value>,
    ) -> Result<CreateFulfillmentResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let reference = data
            .and_then(|data| data.get("reference"))
            .and_then(Value::as_str)
            .unwrap_or("missing-reference");
        if self.ambiguous_after_accept.load(Ordering::SeqCst) {
            return Err(Error::Internal(
                "response lost after courier accepted".into(),
            ));
        }
        Ok(CreateFulfillmentResponse {
            data: Some(json!({"reference": reference, "consignment_id": "sc-123"})),
            labels: vec![CreateFulfillmentLabelInput {
                tracking_number: "TRACK-123".to_string(),
                tracking_url: None,
                label_url: None,
                metadata: None,
            }],
        })
    }

    async fn cancel_fulfillment(&self, _data: Option<&Value>) -> Result<()> {
        Ok(())
    }

    async fn create_return_fulfillment(
        &self,
        _data: Option<&Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Err(Error::Internal(
            "test courier does not create returns".into(),
        ))
    }
}

fn dispatch_input() -> CreateFulfillmentInput {
    CreateFulfillmentInput {
        location_id: None,
        provider_id: "social_test_courier".to_string(),
        shipping_option_id: None,
        delivery_address: None,
        items: vec![CreateFulfillmentItemInput {
            title: "Social order item".to_string(),
            sku: None,
            barcode: None,
            line_item_id: "li_social_test".to_string(),
            inventory_item_id: None,
            quantity: 1,
            metadata: None,
        }],
        labels: None,
        data: Some(json!({"reference": "social-order:ord_dispatch_test"})),
        metadata: Some(json!({"source": "social"})),
    }
}

async fn service_with_test_courier(
    db: &TestDb,
    ambiguous_after_accept: bool,
) -> (FulfillmentService, Arc<SocialTestCourier>) {
    TestDb::execute(
        db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('social_test_courier', 'Social test courier') ON CONFLICT DO NOTHING",
    )
    .await;
    let service = FulfillmentService::new(create_pool(&db.url, 3).expect("pool"));
    let courier = Arc::new(SocialTestCourier {
        calls: AtomicUsize::new(0),
        ambiguous_after_accept: AtomicBool::new(ambiguous_after_accept),
    });
    service.register_provider(courier.clone());
    (service, courier)
}

#[tokio::test]
async fn confirmed_social_dispatch_persists_once_and_replays_the_same_fulfillment() {
    let db = TestDb::migrated().await;
    let (service, courier) = service_with_test_courier(&db, false).await;
    let input = dispatch_input();
    let provider_data = json!({
        "reference": "social-order:ord_dispatch_test",
        "api_key": "test-secret-must-not-persist"
    });

    let first = service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:ord_dispatch_test",
            provider_data.clone(),
        )
        .await
        .expect("first confirmed dispatch");
    let replay = service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:ord_dispatch_test",
            provider_data,
        )
        .await
        .expect("replayed confirmation");

    assert_eq!(first.id, replay.id);
    assert_eq!(first.labels.len(), 1);
    assert_eq!(first.labels[0].tracking_number, "TRACK-123");
    assert_eq!(courier.calls.load(Ordering::SeqCst), 1);
    let found = service
        .lookup_by_reference("social-order:ord_dispatch_test")
        .await
        .expect("lookup")
        .expect("persisted fulfillment");
    assert_eq!(found.id, first.id);
    assert!(!serde_json::to_string(&found)
        .expect("serialize fulfillment")
        .contains("test-secret-must-not-persist"));
    db.drop().await;
}

#[tokio::test]
async fn ambiguous_social_dispatch_blocks_retry_instead_of_creating_a_second_consignment() {
    let db = TestDb::migrated().await;
    let (service, courier) = service_with_test_courier(&db, true).await;
    let input = dispatch_input();
    let provider_data = json!({"reference": "social-order:ord_dispatch_test"});

    let first = service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:ord_dispatch_test",
            provider_data.clone(),
        )
        .await;
    assert!(first.is_err(), "ambiguous provider result must fail closed");
    let retry = service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:ord_dispatch_test",
            provider_data,
        )
        .await;
    assert!(matches!(
        retry,
        Err(Error::Conflict {
            code: "provider_request_unsettled",
            ..
        })
    ));
    assert_eq!(courier.calls.load(Ordering::SeqCst), 1);
    assert!(service
        .lookup_by_reference("social-order:ord_dispatch_test")
        .await
        .expect("lookup")
        .is_none());
    db.drop().await;
}

#[tokio::test]
async fn accepted_reconciliation_finishes_local_writes_without_calling_the_courier_again() {
    let db = TestDb::migrated().await;
    let (service, courier) = service_with_test_courier(&db, true).await;
    let input = dispatch_input();
    let provider_data = json!({"reference": "social-order:ord_dispatch_test"});
    assert!(service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:ord_dispatch_test",
            provider_data.clone(),
        )
        .await
        .is_err());

    service
        .reconcile_unknown_social_dispatch(
            "social-order:ord_dispatch_test",
            "usr_operator",
            "courier-ticket-123",
            SocialDispatchResolution::Accepted {
                courier_reference: "consignment-456".to_string(),
                tracking_number: "TRACK-456".to_string(),
                tracking_url: Some("https://courier.example/track/TRACK-456".to_string()),
                label_url: None,
            },
        )
        .await
        .expect("verified acceptance settles the request");

    let fulfillment = service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:ord_dispatch_test",
            provider_data,
        )
        .await
        .expect("replay persists the confirmed courier result locally");
    assert_eq!(courier.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fulfillment.labels[0].tracking_number, "TRACK-456");
    assert_eq!(
        fulfillment.labels[0].tracking_url.as_deref(),
        Some("https://courier.example/track/TRACK-456")
    );
    assert!(fulfillment
        .data
        .as_ref()
        .is_some_and(|data| data["consignment_id"] == "consignment-456"));
    db.drop().await;
}

#[tokio::test]
async fn verified_not_created_reconciliation_claims_the_same_row_for_one_retry() {
    let db = TestDb::migrated().await;
    let (service, courier) = service_with_test_courier(&db, true).await;
    let input = dispatch_input();
    let provider_data = json!({"reference": "social-order:ord_dispatch_test"});
    assert!(service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:ord_dispatch_test",
            provider_data.clone(),
        )
        .await
        .is_err());

    service
        .reconcile_unknown_social_dispatch(
            "social-order:ord_dispatch_test",
            "usr_operator",
            "courier-ticket-456",
            SocialDispatchResolution::NotCreated,
        )
        .await
        .expect("verified absence allows one retry");
    courier
        .ambiguous_after_accept
        .store(false, Ordering::SeqCst);

    let first_retry = service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:ord_dispatch_test",
            provider_data.clone(),
        )
        .await
        .expect("the claimed request can retry after confirmed absence");
    let replay = service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:ord_dispatch_test",
            provider_data,
        )
        .await
        .expect("replay reuses the accepted retry");
    assert_eq!(first_retry.id, replay.id);
    assert_eq!(courier.calls.load(Ordering::SeqCst), 2);
    db.drop().await;
}

#[tokio::test]
async fn concurrent_reconciliation_allows_only_one_resolution() {
    let db = TestDb::migrated().await;
    let (service, _) = service_with_test_courier(&db, true).await;
    let input = dispatch_input();
    let provider_data = json!({"reference": "social-order:ord_dispatch_test"});
    assert!(service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:ord_dispatch_test",
            provider_data,
        )
        .await
        .is_err());

    let accepted = SocialDispatchResolution::Accepted {
        courier_reference: "consignment-789".to_string(),
        tracking_number: "TRACK-789".to_string(),
        tracking_url: None,
        label_url: None,
    };
    let (first, second) = tokio::join!(
        service.reconcile_unknown_social_dispatch(
            "social-order:ord_dispatch_test",
            "usr_operator_a",
            "courier-ticket-a",
            accepted.clone(),
        ),
        service.reconcile_unknown_social_dispatch(
            "social-order:ord_dispatch_test",
            "usr_operator_b",
            "courier-ticket-b",
            accepted,
        )
    );
    assert_ne!(first.is_ok(), second.is_ok());
    db.drop().await;
}
