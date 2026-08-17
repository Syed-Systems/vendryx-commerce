use async_trait::async_trait;
use rust_decimal::Decimal;
use serde_json::json;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_types::Result;

struct MockLifecycleCarrier;

#[async_trait]
impl FulfillmentProvider for MockLifecycleCarrier {
    fn identifier(&self) -> &'static str {
        "lifecycle-carrier"
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
        _data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Ok(CreateFulfillmentResponse {
            data: Some(json!({ "shipment_id": "shp_991" })),
            labels: vec![CreateFulfillmentLabelInput {
                tracking_number: "TRACK-991-LIFECYCLE".to_string(),
                tracking_url: Some("https://track.example.com/TRACK-991".to_string()),
                label_url: Some("https://labels.example.com/TRACK-991.pdf".to_string()),
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
}

#[tokio::test]
async fn test_fulfillment_creation_items_labels_shipping_and_cancel_lifecycle() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    service.register_provider(Arc::new(MockLifecycleCarrier));

    // Seed dummy provider in Postgres table to satisfy foreign key
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('lifecycle-carrier', 'Lifecycle Carrier') ON CONFLICT DO NOTHING",
    )
    .await;

    // 1. Create fulfillment with 2 line items
    let fulfillment = service
        .create_fulfillment(CreateFulfillmentInput {
            location_id: Some("loc_wh_1".to_string()),
            provider_id: "lifecycle-carrier".to_string(),
            shipping_option_id: None,
            delivery_address: Some(CreateAddressInput {
                first_name: Some("Bob".to_string()),
                last_name: Some("Miller".to_string()),
                address_1: Some("742 Evergreen Terrace".to_string()),
                city: Some("Springfield".to_string()),
                country_code: Some("us".to_string()),
                postal_code: Some("97477".to_string()),
                ..Default::default()
            }),
            items: vec![
                CreateFulfillmentItemInput {
                    title: "Running Shoes".to_string(),
                    sku: Some("SHOE-RUN-42".to_string()),
                    barcode: None,
                    line_item_id: "li_shoe".to_string(),
                    inventory_item_id: Some("inv_shoe".to_string()),
                    quantity: 1,
                    metadata: None,
                },
                CreateFulfillmentItemInput {
                    title: "Running Socks".to_string(),
                    sku: Some("SOCK-RUN-L".to_string()),
                    barcode: None,
                    line_item_id: "li_sock".to_string(),
                    inventory_item_id: Some("inv_sock".to_string()),
                    quantity: 2,
                    metadata: None,
                },
            ],
            labels: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("fulfillment created");

    assert_eq!(fulfillment.items.len(), 2);
    assert_eq!(fulfillment.labels.len(), 1);
    assert_eq!(fulfillment.labels[0].tracking_number, "TRACK-991-LIFECYCLE");
    assert!(fulfillment.delivery_address.is_some());
    assert_eq!(
        fulfillment.delivery_address.as_ref().unwrap().first_name,
        Some("Bob".to_string())
    );
    assert!(fulfillment.shipped_at.is_none());

    // 2. Mark as shipped
    let shipped = service
        .mark_as_shipped(&fulfillment.id)
        .await
        .expect("marked shipped");
    assert!(shipped.shipped_at.is_some());
    assert!(shipped.delivered_at.is_none());

    // 3. Mark as delivered
    let delivered = service
        .mark_as_delivered(&fulfillment.id)
        .await
        .expect("marked delivered");
    assert!(delivered.delivered_at.is_some());

    // 4. Cancel fulfillment
    let canceled = service
        .cancel_fulfillment(&fulfillment.id)
        .await
        .expect("canceled fulfillment");
    assert!(canceled.canceled_at.is_some());

    // 5. Attempt second cancel fails with conflict
    let double_cancel = service.cancel_fulfillment(&fulfillment.id).await;
    assert!(double_cancel.is_err());

    db.drop().await;
}
