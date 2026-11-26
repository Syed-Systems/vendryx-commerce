//! P37-26: two distinct shipments must not collapse onto one idempotency key.
//!
//! `create_fulfillment` keyed the outbound ledger on
//! `create_fulfillment:{provider_id}:{shipping_option_id or "no_reference"}`. Every call
//! without a shipping option therefore shared the constant `no_reference`, so the second
//! genuinely different parcel for a courier was refused as a replay with
//! `idempotency_key_payload_mismatch`.
//!
//! The intent recorded above that code is the opposite: "the caller must supply one to get
//! dedupe rather than merely a durable record." Absent a caller reference the calls should
//! lose *dedupe* and keep the durable record — not share one key. A merchant shipping two
//! parcels through one courier is the ordinary case, not a retry.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_types::{Error, Result};

struct SilentCarrier {
    id: &'static str,
    calls: Option<Arc<AtomicUsize>>,
    fail: bool,
    labels: Vec<CreateFulfillmentLabelInput>,
}

#[async_trait]
impl FulfillmentProvider for SilentCarrier {
    fn identifier(&self) -> &'static str {
        self.id
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![])
    }

    async fn validate_option(&self, _option_data: Option<&serde_json::Value>) -> Result<bool> {
        Ok(true)
    }

    async fn calculate_price(
        &self,
        _option_data: Option<&serde_json::Value>,
        _data: Option<&serde_json::Value>,
        _context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        Ok(Decimal::ZERO)
    }

    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        if let Some(calls) = &self.calls {
            calls.fetch_add(1, Ordering::SeqCst);
        }
        if self.fail {
            return Err(Error::Conflict {
                code: "ambiguous_provider_result",
                message: "provider accepted the request but its response was lost".to_string(),
            });
        }
        Ok(CreateFulfillmentResponse {
            data: data.cloned(),
            labels: self.labels.clone(),
        })
    }

    async fn cancel_fulfillment(&self, _data: Option<&serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn create_return_fulfillment(
        &self,
        _data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Ok(CreateFulfillmentResponse {
            data: None,
            labels: vec![],
        })
    }
}

fn parcel(reference: &str) -> CreateFulfillmentInput {
    CreateFulfillmentInput {
        location_id: Some("loc_wh_1".to_string()),
        provider_id: "silent-carrier".to_string(),
        shipping_option_id: None,
        delivery_address: None,
        items: vec![CreateFulfillmentItemInput {
            title: format!("Parcel {reference}"),
            sku: Some(reference.to_string()),
            barcode: None,
            line_item_id: format!("li_{reference}"),
            inventory_item_id: None,
            quantity: 1,
            metadata: None,
        }],
        labels: None,
        data: Some(json!({ "reference": reference })),
        metadata: None,
    }
}

#[tokio::test]
async fn two_parcels_for_one_courier_are_both_created() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: false,
        labels: vec![],
    }));

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    let first = service
        .create_fulfillment(parcel("PARCEL_A"))
        .await
        .expect("the first parcel is created");

    // This is the assertion that failed before P37-26: the second call reused the first
    // call's key with a different payload and was rejected as a replay.
    let second = service
        .create_fulfillment(parcel("PARCEL_B"))
        .await
        .expect("a second, different parcel must not be refused as a replay");

    assert_ne!(
        first.id, second.id,
        "two distinct shipments must be two distinct fulfillments"
    );

    // Both are durably recorded and independently resolvable — losing dedupe must not
    // mean losing the ledger.
    for (reference, expected) in [("PARCEL_A", &first.id), ("PARCEL_B", &second.id)] {
        let found = service
            .lookup_by_reference(reference)
            .await
            .expect("lookup executes")
            .unwrap_or_else(|| panic!("{reference} must resolve"));
        assert_eq!(&found.id, expected);
    }
}

#[tokio::test]
async fn repeated_order_derived_key_returns_the_first_local_fulfillment() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: false,
        labels: vec![],
    }));

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    let first = service
        .create_fulfillment_idempotent(parcel("SOCIAL_ORDER_123"), "social-order:order_123")
        .await
        .expect("first dispatch succeeds");
    let changed = service
        .create_fulfillment_idempotent(parcel("CHANGED_ORDER_PAYLOAD"), "social-order:order_123")
        .await;
    assert!(
        changed.is_err(),
        "a key cannot authorize a changed shipment payload"
    );
    let replay = service
        .create_fulfillment_idempotent(parcel("SOCIAL_ORDER_123"), "social-order:order_123")
        .await
        .expect("retry returns the first dispatch");

    assert_eq!(first.id, replay.id);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn stable_order_key_cannot_switch_courier_after_dispatch_started() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let first_calls = Arc::new(AtomicUsize::new(0));
    let second_calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(first_calls.clone()),
        fail: false,
        labels: vec![],
    }));
    service.register_provider(Arc::new(SilentCarrier {
        id: "second-carrier",
        calls: Some(second_calls.clone()),
        fail: false,
        labels: vec![],
    }));
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier'), ('second-carrier', 'Second Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    service
        .create_fulfillment_idempotent(
            parcel("SOCIAL_ORDER_PROVIDER_BOUND"),
            "social-order:provider_bound",
        )
        .await
        .expect("first courier dispatch succeeds");
    let mut changed_courier = parcel("SOCIAL_ORDER_PROVIDER_BOUND");
    changed_courier.provider_id = "second-carrier".to_string();
    assert!(
        service
            .create_fulfillment_idempotent(changed_courier, "social-order:provider_bound")
            .await
            .is_err()
    );

    assert_eq!(first_calls.load(Ordering::SeqCst), 1);
    assert_eq!(second_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn tenant_credentials_are_available_for_dispatch_but_not_persisted() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: None,
        fail: false,
        labels: vec![],
    }));

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    let mut input = parcel("SOCIAL_ORDER_SECRET_TEST");
    input.data = Some(json!({
        "reference": "SOCIAL_ORDER_SECRET_TEST",
        "credentials": {"api_key": "input-api-secret"}
    }));
    let provider_data = json!({
        "reference": "SOCIAL_ORDER_SECRET_TEST",
        "api_key": "tenant-api-secret",
        "client_secret": "tenant-client-secret",
        "provider_options": {
            "Authorization": "nested-authorization-secret",
            "credentials": {"refresh_token": "nested-refresh-secret"}
        },
        "recipient_name": "Test Buyer",
    });
    let fulfillment = service
        .create_fulfillment_idempotent_with_provider_data(
            input.clone(),
            "social-order:secret_test",
            provider_data.clone(),
        )
        .await
        .expect("tenant-scoped courier call succeeds");

    input.data.as_mut().unwrap()["credentials"]["api_key"] = json!("rotated-input-api-secret");
    let replay = service
        .create_fulfillment_idempotent_with_provider_data(
            input,
            "social-order:secret_test",
            json!({
                "reference": "SOCIAL_ORDER_SECRET_TEST",
                "api_key": "rotated-tenant-api-secret",
                "client_secret": "rotated-tenant-client-secret",
                "provider_options": {
                    "Authorization": "rotated-nested-authorization-secret",
                    "credentials": {"refresh_token": "rotated-nested-refresh-secret"}
                },
                "recipient_name": "Test Buyer",
            }),
        )
        .await
        .expect("secret rotation does not alter request identity or cause a second dispatch");

    let persisted = serde_json::to_string(&fulfillment).expect("fulfillment JSON");
    assert_eq!(fulfillment.id, replay.id);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(!persisted.contains("tenant-api-secret"));
    assert!(!persisted.contains("tenant-client-secret"));
    assert!(!persisted.contains("api_key"));
    assert!(!persisted.contains("client_secret"));
    assert!(!persisted.contains("nested-authorization-secret"));
    assert!(!persisted.contains("nested-refresh-secret"));
    assert!(!persisted.contains("Authorization"));
    assert!(!persisted.contains("refresh_token"));
    assert!(!persisted.contains("input-api-secret"));
}

#[tokio::test]
async fn retry_repairs_local_item_write_after_provider_success_without_redispatch() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: false,
        labels: vec![],
    }));
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;
    TestDb::execute(
        &db,
        "CREATE FUNCTION fulfillment.fail_social_item_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.line_item_id = 'li_SOCIAL_RETRY' THEN RAISE EXCEPTION 'injected local item failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER fail_social_item_insert BEFORE INSERT ON fulfillment.fulfillment_item FOR EACH ROW EXECUTE FUNCTION fulfillment.fail_social_item_insert()",
    )
    .await;

    let mut input = parcel("SOCIAL_RETRY");
    input.items[0].line_item_id = "li_SOCIAL_RETRY".to_string();
    input.delivery_address = Some(CreateAddressInput {
        first_name: Some("Test Buyer".to_string()),
        address_1: Some("House 1".to_string()),
        ..Default::default()
    });
    assert!(
        service
            .create_fulfillment_idempotent(input.clone(), "social-order:retry_local_write")
            .await
            .is_err()
    );
    let fulfillment_id = service
        .lookup_by_reference("SOCIAL_RETRY")
        .await
        .expect("local fulfillment lookup")
        .expect("fulfillment row survived the injected child failure")
        .id;

    TestDb::execute(
        &db,
        "DROP TRIGGER fail_social_item_insert ON fulfillment.fulfillment_item; DROP FUNCTION fulfillment.fail_social_item_insert()",
    )
    .await;
    let retry = service
        .create_fulfillment_idempotent(input, "social-order:retry_local_write")
        .await
        .expect("retry repairs the missing child without repeating the courier call");

    assert_eq!(retry.id, fulfillment_id);
    assert_eq!(retry.items.len(), 1);
    assert!(retry.delivery_address.is_some());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_reuses_address_when_fulfillment_row_insert_failed() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: false,
        labels: vec![],
    }));
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;
    TestDb::execute(
        &db,
        "CREATE FUNCTION fulfillment.fail_social_fulfillment_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected local fulfillment failure'; END; $$; CREATE TRIGGER fail_social_fulfillment_insert BEFORE INSERT ON fulfillment.fulfillment FOR EACH ROW EXECUTE FUNCTION fulfillment.fail_social_fulfillment_insert()",
    )
    .await;

    let mut input = parcel("SOCIAL_ADDRESS_RETRY");
    input.delivery_address = Some(CreateAddressInput {
        first_name: Some("Test Buyer".to_string()),
        address_1: Some("House 2".to_string()),
        ..Default::default()
    });
    assert!(
        service
            .create_fulfillment_idempotent(input.clone(), "social-order:retry_address")
            .await
            .is_err()
    );
    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.address WHERE address_1 = 'House 2'")
            .await,
        1
    );

    TestDb::execute(
        &db,
        "DROP TRIGGER fail_social_fulfillment_insert ON fulfillment.fulfillment; DROP FUNCTION fulfillment.fail_social_fulfillment_insert()",
    )
    .await;
    let retry = service
        .create_fulfillment_idempotent(input, "social-order:retry_address")
        .await
        .expect("retry finishes local write with original courier result");

    assert_eq!(
        db.count("SELECT count(*) FROM fulfillment.address WHERE address_1 = 'House 2'")
            .await,
        1
    );
    assert!(retry.delivery_address.is_some());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_repairs_local_label_write_after_provider_success_without_redispatch() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: false,
        labels: vec![CreateFulfillmentLabelInput {
            tracking_number: "TRACKING_LABEL_RETRY".to_string(),
            tracking_url: Some("https://carrier.example/track/TRACKING_LABEL_RETRY".to_string()),
            label_url: None,
            metadata: Some(json!({"carrier_status": "accepted"})),
        }],
    }));
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;
    TestDb::execute(
        &db,
        "CREATE FUNCTION fulfillment.fail_social_label_insert() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.tracking_number = 'TRACKING_LABEL_RETRY' THEN RAISE EXCEPTION 'injected local label failure'; END IF; RETURN NEW; END; $$; CREATE TRIGGER fail_social_label_insert BEFORE INSERT ON fulfillment.fulfillment_label FOR EACH ROW EXECUTE FUNCTION fulfillment.fail_social_label_insert()",
    )
    .await;

    let input = parcel("SOCIAL_LABEL_RETRY");
    assert!(
        service
            .create_fulfillment_idempotent(input.clone(), "social-order:retry_label")
            .await
            .is_err()
    );
    let fulfillment_id = service
        .lookup_by_reference("SOCIAL_LABEL_RETRY")
        .await
        .expect("local fulfillment lookup")
        .expect("fulfillment row survived the injected label failure")
        .id;

    TestDb::execute(
        &db,
        "DROP TRIGGER fail_social_label_insert ON fulfillment.fulfillment_label; DROP FUNCTION fulfillment.fail_social_label_insert()",
    )
    .await;
    let retry = service
        .create_fulfillment_idempotent(input, "social-order:retry_label")
        .await
        .expect("retry persists the cached courier label without a second dispatch");

    assert_eq!(retry.id, fulfillment_id);
    assert_eq!(retry.labels.len(), 1);
    assert_eq!(retry.labels[0].tracking_number, "TRACKING_LABEL_RETRY");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn ambiguous_provider_result_is_not_retried_as_a_second_consignment() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    let calls = Arc::new(AtomicUsize::new(0));
    service.register_provider(Arc::new(SilentCarrier {
        id: "silent-carrier",
        calls: Some(calls.clone()),
        fail: true,
        labels: vec![],
    }));
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('silent-carrier', 'Silent Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    let input = parcel("AMBIGUOUS_SOCIAL_ORDER");
    assert!(
        service
            .create_fulfillment_idempotent(input.clone(), "social-order:ambiguous")
            .await
            .is_err()
    );
    let retry = service
        .create_fulfillment_idempotent(input, "social-order:ambiguous")
        .await;

    assert!(
        retry.is_err(),
        "an unsettled provider result must stay held"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
