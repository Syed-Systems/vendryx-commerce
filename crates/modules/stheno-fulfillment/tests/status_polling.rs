//! P23-10: Polling fallback for couriers that do not push, sharing the webhook reducer.

use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::{Arc, Mutex};
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant};
use stheno_fulfillment::*;
use stheno_types::Result;

const TENANT: &str = TestDb::TEST_TENANT;

struct PollingCarrier {
    status: Mutex<CourierStatus>,
}

impl PollingCarrier {
    fn new(initial: CourierStatus) -> Self {
        Self {
            status: Mutex::new(initial),
        }
    }

    fn set_status(&self, s: CourierStatus) {
        *self.status.lock().unwrap() = s;
    }
}

#[async_trait]
impl FulfillmentProvider for PollingCarrier {
    fn identifier(&self) -> &'static str {
        "polling_carrier"
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
                tracking_number: "POLL-TRK-777".to_string(),
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
        Ok(self.status.lock().unwrap().clone())
    }
}

#[tokio::test]
async fn polling_updates_fulfillment_via_shared_reducer() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    let carrier = Arc::new(PollingCarrier::new(CourierStatus::InTransit));
    service.register_provider(carrier.clone());

    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('polling_carrier', 'Polling Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    tenant::scope(TENANT.to_string(), {
        let service = service.clone();
        let carrier = carrier.clone();
        async move {
            let created = service
                .create_fulfillment(CreateFulfillmentInput {
                    location_id: None,
                    provider_id: "polling_carrier".to_string(),
                    shipping_option_id: None,
                    delivery_address: None,
                    items: vec![CreateFulfillmentItemInput {
                        title: "Test Item".to_string(),
                        sku: None,
                        barcode: None,
                        line_item_id: "li_1".to_string(),
                        inventory_item_id: None,
                        quantity: 1,
                        metadata: None,
                    }],
                    labels: None,
                    data: Some(json!({ "consignment_id": "POLL-TRK-777" })),
                    metadata: None,
                })
                .await
                .expect("create fulfillment");

            assert!(created.shipped_at.is_none());
            assert!(created.delivered_at.is_none());

            // 1. Poll while in transit -> sets shipped_at
            let polled1 = service
                .poll_consignment_status("polling_carrier", "POLL-TRK-777")
                .await
                .expect("poll succeeds")
                .expect("fulfillment found");

            assert!(polled1.shipped_at.is_some());
            assert!(polled1.delivered_at.is_none());
            assert_eq!(
                polled1
                    .data
                    .as_ref()
                    .and_then(|d| d.get("courier_status"))
                    .and_then(|s| s.as_str()),
                Some("in_transit")
            );

            // 2. Carrier updates to delivered -> poll again -> sets delivered_at
            carrier.set_status(CourierStatus::Delivered);

            let polled2 = service
                .poll_consignment_status("polling_carrier", "POLL-TRK-777")
                .await
                .expect("poll succeeds")
                .expect("fulfillment found");

            assert!(polled2.delivered_at.is_some());
            assert_eq!(
                polled2
                    .data
                    .as_ref()
                    .and_then(|d| d.get("courier_status"))
                    .and_then(|s| s.as_str()),
                Some("delivered")
            );
        }
    })
    .await;

    db.drop().await;
}
