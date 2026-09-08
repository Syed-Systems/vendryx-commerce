//! P23-11: [BP] Unknown status is loud: an unmapped status raises an alert and leaves
//! the fulfillment in a named state, never a silent no-op.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant};
use stheno_fulfillment::*;
use stheno_types::Result;

const TENANT: &str = TestDb::TEST_TENANT;

struct AlienCarrier;

#[async_trait]
impl FulfillmentProvider for AlienCarrier {
    fn identifier(&self) -> &'static str {
        "alien_carrier"
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
            labels: vec![CreateFulfillmentLabelInput {
                tracking_number: "ALIEN-TRK-999".to_string(),
                tracking_url: None,
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
        Ok(CreateFulfillmentResponse {
            data: None,
            labels: vec![],
        })
    }

    async fn get_tracking_status(&self, _reference: &str) -> Result<CourierStatus> {
        // Returns completely unknown status
        Ok(CourierStatus::parse("orbital_quarantine_retained"))
    }
}

#[tokio::test]
async fn unknown_status_is_loud_and_leaves_named_state() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    let carrier = Arc::new(AlienCarrier);
    service.register_provider(carrier);

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('alien_carrier', 'Alien Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    tenant::scope(TENANT.to_string(), {
        let service = service.clone();
        async move {
            let _created = service
                .create_fulfillment(CreateFulfillmentInput {
                    location_id: None,
                    provider_id: "alien_carrier".to_string(),
                    shipping_option_id: None,
                    delivery_address: None,
                    items: vec![CreateFulfillmentItemInput {
                        title: "Alien Package".to_string(),
                        sku: None,
                        barcode: None,
                        line_item_id: "li_alien".to_string(),
                        inventory_item_id: None,
                        quantity: 1,
                        metadata: None,
                    }],
                    labels: None,
                    data: Some(json!({ "consignment_id": "ALIEN-TRK-999" })),
                    metadata: None,
                })
                .await
                .expect("create fulfillment");

            // 1. Direct apply_status_update with unknown status
            let updated = service
                .apply_status_update(
                    "ALIEN-TRK-999",
                    "alien_carrier",
                    "custom_warehouse_exception_code_99",
                )
                .await
                .expect("status update succeeds")
                .expect("fulfillment found");

            let data = updated.data.expect("fulfillment has data");
            // Must raise an alert
            assert_eq!(
                data.get("unknown_status_alert").and_then(|v| v.as_bool()),
                Some(true),
                "unmapped status must raise an alert"
            );
            assert!(
                data.get("alert_reason").is_some(),
                "alert reason must be recorded"
            );
            // Must be left in the named state, NOT discarded or set to default
            assert_eq!(
                data.get("courier_status").and_then(|s| s.as_str()),
                Some("custom_warehouse_exception_code_99")
            );
            assert_eq!(
                data.get("courier_raw_status").and_then(|s| s.as_str()),
                Some("custom_warehouse_exception_code_99")
            );

            // 2. Via polling with provider returning unmapped status
            let polled = service
                .poll_consignment_status("alien_carrier", "ALIEN-TRK-999")
                .await
                .expect("poll succeeds")
                .expect("fulfillment found");

            let polled_data = polled.data.expect("polled fulfillment has data");
            assert_eq!(
                polled_data
                    .get("unknown_status_alert")
                    .and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(
                polled_data.get("courier_status").and_then(|s| s.as_str()),
                Some("orbital_quarantine_retained")
            );
        }
    })
    .await;

    db.drop().await;
}
