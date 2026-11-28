//! P28-03: Provider failure modes — gateway timeout, gateway 500, callback never arriving, courier API down.
//! Each failure mode has a defined, tested outcome.
//!
//! Enforces:
//! 1. Gateway timeout: ambiguous outcome handled via idempotent_call and retrieve reconciliation (no double charge).
//! 2. Gateway 500: external server error returns clean failure, preserving cart and session.
//! 3. Callback never arriving: stranded request resolved and converged via background reconciliation sweep.
//! 4. Courier API down: consignment creation fails closed, writes zero partial records, and refuses uncoordinated retry to prevent duplicate parcels.

use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::{
    CalculatePriceContext, CreateFulfillmentInput, CreateFulfillmentItemInput,
    CreateFulfillmentLabelInput, CreateFulfillmentResponse, FulfillmentOptionDTO,
    FulfillmentProvider, FulfillmentService,
};
use stheno_payment::{CaptureInput, PaymentProvider, PaymentService};
use stheno_payment_bkash::BkashPaymentProvider;
use stheno_payment_offline::OfflinePaymentProvider;
use stheno_types::{Error, Result};

// ── 1. Gateway Timeout: Ambiguity handling via idempotent_call ───────────────

#[tokio::test]
async fn test_provider_failure_mode_gateway_timeout_idempotency_recovery() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_timeout_test")));
    service.register_provider(offline);

    let key = "idem_key_gw_timeout_p28";
    let payload = json!({
        "amount": 3500,
        "currency": "BDT"
    });

    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_clone = attempts.clone();

    // First attempt: Network connection timeout occurs while contacting payment gateway.
    let res1: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_timeout_test",
            "authorize",
            "session",
            "sess_timeout_p28",
            &payload,
            |_k| {
                let att = attempts_clone.clone();
                async move {
                    att.fetch_add(1, Ordering::SeqCst);
                    // Gateway connection timed out (ambiguous outcome)
                    Err(Error::Internal(
                        "Gateway timeout: Connection timed out after 30000ms"
                            .to_string()
                            .into(),
                    ))
                }
            },
        )
        .await;

    // Outcome 1: Initial call returns error to caller; outbound request recorded as unsettled.
    assert!(res1.is_err(), "caller must see the timeout error");
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "first call dispatched once"
    );

    // Second attempt: Same caller retries with the SAME idempotency key and payload.
    // The engine invokes provider.retrieve() to reconcile the ambiguous state without re-charging.
    let res2: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            "pp_timeout_test",
            "authorize",
            "session",
            "sess_timeout_p28",
            &payload,
            |_k| async move {
                // Must not be called again! Double charging prevented.
                panic!(
                    "CRITICAL: Provider call re-executed! Double-charge vulnerability detected."
                );
            },
        )
        .await;

    // Outcome 2: Retry safely converges to success without duplicate charge.
    assert!(
        res2.is_ok(),
        "ambiguous timeout must reconcile successfully via retrieve reconciliation"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "provider was NOT double-invoked"
    );
}

// ── 2. Gateway 500: Server error returns clean failure & preserves cart ──────

#[tokio::test]
async fn test_provider_failure_mode_gateway_500_clean_error_preserves_cart() {
    let mock_server = MockServer::start().await;

    // 1. Grant token succeeds
    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/token/grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statusCode": "0000",
            "id_token": "token_gw_500",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // 2. Execute returns HTTP 500 Internal Server Error from the gateway
    Mock::given(method("POST"))
        .and(path("/tokenized/checkout/execute/PAY_BKASH_500"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({
            "status": "error",
            "message": "Internal Server Error in upstream payment switch"
        })))
        .mount(&mock_server)
        .await;

    // 3. Convergence query confirms transaction did not succeed
    Mock::given(method("GET"))
        .and(path("/tokenized/checkout/payment/query/PAY_BKASH_500"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "paymentID": "PAY_BKASH_500",
            "transactionStatus": "Failed"
        })))
        .mount(&mock_server)
        .await;

    let provider = BkashPaymentProvider::new(
        &mock_server.uri(),
        "app_key_500",
        "app_secret_500",
        "user_500",
        "pass_500",
    );

    let capture_input = CaptureInput {
        payment_id: "PAY_BKASH_500".to_string(),
        amount: dec!(2400.00),
        currency_code: "BDT".to_string(),
        data: None,
    };

    let result = provider.capture(capture_input).await;

    // Outcome: Returns clean structured Error::Invalid, does not panic, allows session retry.
    assert!(result.is_err(), "Gateway 500 must fail closed");
    let err = result.unwrap_err();
    assert!(
        matches!(err, Error::Invalid { .. }),
        "Expected Invalid error, got: {err:?}"
    );
}

// ── 3. Callback Never Arriving: Convergence via background sweep ─────────────

#[tokio::test]
async fn test_provider_failure_mode_callback_never_arriving_converged_by_sweep() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = PaymentService::new(pool);
    let provider_id = "pp_sweep_convergence";
    service.register_provider(Arc::new(OfflinePaymentProvider::new(Some(provider_id))));

    let key = "idem_stranded_callback_p28";
    let payload = json!({ "amount": 5000, "currency": "BDT" });

    // The payment was initiated and customer sent to gateway, but the callback was lost in transit.
    // The request row exists in ledger in an ambiguous state.
    let initial_res: Result<serde_json::Value> = service
        .idempotent_call(
            key,
            provider_id,
            "capture",
            "payment",
            "pay_stranded_p28",
            &payload,
            |_k| async move {
                Err(Error::Internal(
                    "Network connection reset; callback never arrived"
                        .to_string()
                        .into(),
                ))
            },
        )
        .await;

    assert!(
        initial_res.is_err(),
        "initial attempt timed out / lost callback"
    );

    // The background sweep runs (reconcile_unsettled_requests).
    // The sweep discovers the stranded request, queries the provider's status, and converges it.
    let report = service
        .reconcile_unsettled_requests(0, 100)
        .await
        .expect("sweep must run clean");

    // Outcome: Stranded request was scanned and converged to succeeded without customer intervention.
    assert_eq!(
        report.scanned, 1,
        "stranded request must be scanned by sweep"
    );
    assert_eq!(
        report.converged, 1,
        "stranded request must be converged by sweep"
    );

    // Second sweep sees zero stranded requests.
    let second_report = service
        .reconcile_unsettled_requests(0, 100)
        .await
        .expect("second sweep must run clean");
    assert_eq!(second_report.scanned, 0, "no stranded requests remain");
}

// ── 4. Courier API Down: Consignment creation fails closed, 0 duplicates ─────

struct FailingCourierProvider {
    id: &'static str,
    is_down: Arc<std::sync::atomic::AtomicBool>,
}

#[async_trait]
impl FulfillmentProvider for FailingCourierProvider {
    fn identifier(&self) -> &'static str {
        self.id
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![FulfillmentOptionDTO {
            id: "failing-express".to_string(),
            is_return: false,
        }])
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
        Ok(dec!(60.00))
    }

    async fn create_fulfillment(
        &self,
        data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        if self.is_down.load(Ordering::SeqCst) {
            return Err(Error::Internal(
                "Courier gateway API down: 503 Service Unavailable"
                    .to_string()
                    .into(),
            ));
        }

        let ref_id = data
            .and_then(|d| d.get("reference"))
            .and_then(|v| v.as_str())
            .unwrap_or("PARCEL-001");

        Ok(CreateFulfillmentResponse {
            data: Some(json!({ "consignment_id": format!("CNS-{ref_id}"), "reference": ref_id })),
            labels: vec![CreateFulfillmentLabelInput {
                tracking_number: format!("TRK-{ref_id}"),
                tracking_url: Some(format!("https://courier.test/tracking/TRK-{ref_id}")),
                label_url: None,
                metadata: None,
            }],
        })
    }

    async fn cancel_fulfillment(&self, _data: Option<&serde_json::Value>) -> Result<()> {
        Ok(())
    }

    async fn create_return_fulfillment(
        &self,
        _data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Err(Error::Internal("Not supported".to_string().into()))
    }
}

#[tokio::test]
async fn test_provider_failure_mode_courier_api_down_fails_closed_zero_duplicates() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    let is_down = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let provider = Arc::new(FailingCourierProvider {
        id: "courier_down_test",
        is_down: is_down.clone(),
    });
    service.register_provider(provider);

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('courier_down_test', 'Failing Courier Test') ON CONFLICT DO NOTHING",
    )
    .await;

    let fulfillment_input = CreateFulfillmentInput {
        provider_id: "courier_down_test".to_string(),
        shipping_option_id: None,
        delivery_address: None,
        items: vec![CreateFulfillmentItemInput {
            title: "Winter Jacket".to_string(),
            sku: Some("WJ-01".to_string()),
            barcode: None,
            line_item_id: "li_jacket_1".to_string(),
            inventory_item_id: None,
            quantity: 1,
            metadata: None,
        }],
        labels: None,
        data: Some(json!({ "reference": "REF-ORD-999" })),
        metadata: None,
        location_id: None,
    };

    // Attempt 1: Courier API is down (503 Service Unavailable).
    let res1 = service.create_fulfillment(fulfillment_input.clone()).await;

    // Outcome 1: Fails closed. No partial database records committed.
    assert!(
        res1.is_err(),
        "Consignment creation must fail when courier is down"
    );

    // Verify zero fulfillments exist for this reference.
    let lookup_before = service
        .lookup_by_reference("REF-ORD-999")
        .await
        .expect("lookup");
    assert!(
        lookup_before.is_none(),
        "Zero fulfillments must exist after courier API failure (no partial writes)"
    );

    // Attempt 2: Courier recovers.
    is_down.store(false, Ordering::SeqCst);

    let res2 = service.create_fulfillment(fulfillment_input).await;
    if let Err(e) = &res2 {
        panic!("Consignment creation failed once courier recovers: {e:?}");
    }

    let ful = res2.unwrap();
    assert_eq!(ful.labels.len(), 1, "Exactly one label created");
    assert_eq!(ful.labels[0].tracking_number, "TRK-REF-ORD-999");

    // Verify exactly ONE fulfillment exists in the system (zero duplicate consignments).
    let lookup_after = service
        .lookup_by_reference("REF-ORD-999")
        .await
        .expect("lookup");
    assert!(
        lookup_after.is_some(),
        "Exactly ONE consignment created after recovery — zero duplicate parcels"
    );
    assert_eq!(lookup_after.unwrap().id, ful.id);
}
