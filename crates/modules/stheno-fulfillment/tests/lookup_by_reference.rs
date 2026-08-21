//! P32-13: Courier lookup by merchant reference.
//! Without it a stranded consignment can only be refused, never resolved.
//!
//! P37-16: this file used to assert only that an unknown reference returns `None`. A
//! `lookup_by_reference` that always returned `None` passed it — which is precisely the
//! function that cannot resolve a stranded parcel. The positive cases are the ones that
//! matter on the phone to a courier, and `lookup_by_reference` matches on three different
//! keys, none of which had a test.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_types::Result;

/// Echoes back the data it was handed, the way a courier confirms the consignment
/// reference it was asked to create. `create_fulfillment` stores
/// `response.data.or(input.data)`, so echoing exercises the provider path rather than
/// the fallback.
///
/// Carries its own identifier so the test can register three of them. That is not
/// decoration: `create_fulfillment` keys the outbound ledger on
/// `provider_id` + `shipping_option_id`, so two consignments for one provider with no
/// shipping option share an idempotency key and the second is refused with
/// `idempotency_key_payload_mismatch`. Three providers is the cheapest way to get three
/// distinct keys without seeding the `shipping_option` FK chain.
struct EchoCarrier(&'static str);

#[async_trait]
impl FulfillmentProvider for EchoCarrier {
    fn identifier(&self) -> &'static str {
        self.0
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

async fn consignment(
    service: &FulfillmentService,
    provider_id: &str,
    data: serde_json::Value,
) -> String {
    service
        .create_fulfillment(CreateFulfillmentInput {
            location_id: Some("loc_wh_1".to_string()),
            provider_id: provider_id.to_string(),
            shipping_option_id: None,
            delivery_address: None,
            items: vec![CreateFulfillmentItemInput {
                title: "Boxed Item".to_string(),
                sku: Some("BOX-1".to_string()),
                barcode: None,
                line_item_id: "li_box".to_string(),
                inventory_item_id: None,
                quantity: 1,
                metadata: None,
            }],
            labels: None,
            data: Some(data),
            metadata: None,
        })
        .await
        .expect("fulfillment is created")
        .id
}

#[tokio::test]
async fn lookup_by_merchant_reference_finds_fulfillment() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    for id in ["echo-pathao", "echo-steadfast", "echo-redx"] {
        service.register_provider(Arc::new(EchoCarrier(id)));
        TestDb::execute(
            &db,
            &format!(
                "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('{id}', '{id}') ON CONFLICT DO NOTHING"
            ),
        )
        .await;
    }

    // An unknown reference resolves to nothing. Kept from the original test.
    let missing = service
        .lookup_by_reference("REF_UNKNOWN_123")
        .await
        .expect("lookup executes");
    assert!(missing.is_none());

    // `lookup_by_reference` accepts three different keys because couriers disagree about
    // what the field is called. Each gets its own consignment, so a lookup that returned
    // the wrong row — or simply the first row — fails on the id assertion rather than
    // passing on `is_some()`.
    let by_reference = consignment(
        &service,
        "echo-pathao",
        json!({ "reference": "REF_PATHAO_001" }),
    )
    .await;
    let by_merchant = consignment(
        &service,
        "echo-steadfast",
        json!({ "merchant_reference": "REF_STEADFAST_002" }),
    )
    .await;
    let by_tracking = consignment(
        &service,
        "echo-redx",
        json!({ "tracking_number": "REF_REDX_003" }),
    )
    .await;

    for (reference, expected) in [
        ("REF_PATHAO_001", &by_reference),
        ("REF_STEADFAST_002", &by_merchant),
        ("REF_REDX_003", &by_tracking),
    ] {
        let found = service
            .lookup_by_reference(reference)
            .await
            .expect("lookup executes")
            .unwrap_or_else(|| panic!("{reference} must resolve to a consignment"));
        assert_eq!(
            &found.id, expected,
            "{reference} resolved to the wrong consignment"
        );
    }

    // A reference that looks like the others but belongs to nobody still resolves to
    // nothing, now that there is real data to accidentally match against.
    assert!(service
        .lookup_by_reference("REF_PATHAO_999")
        .await
        .expect("lookup executes")
        .is_none());
}
