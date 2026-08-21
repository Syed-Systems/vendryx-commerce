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
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_types::Result;

struct SilentCarrier;

#[async_trait]
impl FulfillmentProvider for SilentCarrier {
    fn identifier(&self) -> &'static str {
        "silent-carrier"
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
        Ok(CreateFulfillmentResponse {
            data: data.cloned(),
            labels: vec![],
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
    service.register_provider(Arc::new(SilentCarrier));

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
