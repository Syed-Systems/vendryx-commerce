use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_fulfillment::*;
use stheno_types::Result;

struct MockDynamicCarrier;

#[async_trait]
impl FulfillmentProvider for MockDynamicCarrier {
    fn identifier(&self) -> &'static str {
        "dynamic-carrier"
    }

    async fn get_fulfillment_options(&self) -> Result<Vec<FulfillmentOptionDTO>> {
        Ok(vec![FulfillmentOptionDTO {
            id: "dynamic-rate".to_string(),
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
        context: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        // Base $12 + $3 per kg
        let base = dec!(12.00);
        let weight = context
            .and_then(|c| c.total_weight)
            .unwrap_or(Decimal::ZERO);
        Ok(base + (weight * dec!(3.00)))
    }

    async fn create_fulfillment(
        &self,
        _data: Option<&serde_json::Value>,
        _items: &[CreateFulfillmentItemInput],
        _order_data: Option<&serde_json::Value>,
    ) -> Result<CreateFulfillmentResponse> {
        Ok(CreateFulfillmentResponse {
            data: None,
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

#[tokio::test]
async fn test_calculated_shipping_pricing_through_provider() {
    let db = TestDb::migrated().await;
    TestDb::execute(
        &db,
        "INSERT INTO fulfillment.fulfillment_provider (id, name) VALUES ('dynamic-carrier', 'Dynamic Carrier') ON CONFLICT DO NOTHING",
    )
    .await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);
    service.register_provider(Arc::new(MockDynamicCarrier));

    let set = service
        .create_fulfillment_set(CreateFulfillmentSetInput {
            name: "Express Set".to_string(),
            r#type: "shipping".to_string(),
            metadata: None,
        })
        .await
        .expect("set created");

    let zone = service
        .create_service_zone(CreateServiceZoneInput {
            fulfillment_set_id: set.id.clone(),
            name: "Domestic Zone".to_string(),
            metadata: None,
        })
        .await
        .expect("zone created");

    let profile = service
        .create_shipping_profile(CreateShippingProfileInput {
            name: "Heavy Goods".to_string(),
            r#type: None,
            metadata: None,
        })
        .await
        .expect("profile created");

    // Calculated shipping option
    let calc_option = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Dynamic Freight".to_string(),
            price_type: "calculated".to_string(),
            service_zone_id: zone.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "dynamic-carrier".to_string(),
            shipping_option_type_id: None,
            data: None,
            metadata: None,
        })
        .await
        .expect("calculated option created");

    // Flat shipping option
    let flat_option = service
        .create_shipping_option(CreateShippingOptionInput {
            name: "Flat Rate Standard".to_string(),
            price_type: "flat".to_string(),
            service_zone_id: zone.id.clone(),
            shipping_profile_id: profile.id.clone(),
            provider_id: "manual".to_string(),
            shipping_option_type_id: None,
            data: Some(serde_json::json!({ "amount": "15.50" })),
            metadata: None,
        })
        .await
        .expect("flat option created");

    // 1. Calculate price with 5kg -> $12 + (5 * $3) = $27.00
    let price_5kg = service
        .calculate_shipping_price(
            &calc_option.id,
            Some(&CalculatePriceContext {
                currency_code: "usd".to_string(),
                total_weight: Some(dec!(5.0)),
                items_count: 2,
            }),
        )
        .await
        .expect("price calculated for 5kg");

    assert_eq!(price_5kg, dec!(27.00));

    // 2. Calculate flat rate price -> $15.50
    let flat_price = service
        .calculate_shipping_price(&flat_option.id, None)
        .await
        .expect("flat price calculated");

    assert_eq!(flat_price, dec!(15.50));

    db.drop().await;
}
