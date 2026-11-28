//! P23-06: stheno-fulfillment-steadfast crate: options, rates, consignment, cancel, return.

use rust_decimal_macros::dec;
use serde_json::json;
use stheno_fulfillment::{CalculatePriceContext, CreateFulfillmentItemInput, FulfillmentProvider};
use stheno_fulfillment_steadfast::{SteadfastConfig, SteadfastFulfillmentProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn steadfast_options_and_zone_rates() {
    let provider = SteadfastFulfillmentProvider::default();
    assert_eq!(provider.identifier(), "steadfast");

    let options = provider.get_fulfillment_options().await.expect("options");
    assert_eq!(options.len(), 4);
    assert!(options.iter().any(|o| o.id == "steadfast-standard"));
    assert!(options
        .iter()
        .any(|o| o.id == "steadfast-return" && o.is_return));

    // Zone rates
    let inside_rate = provider
        .calculate_price(
            Some(&json!({ "tier": "inside_city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(1.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("inside rate");
    assert_eq!(inside_rate, dec!(60.00));

    let sub_rate = provider
        .calculate_price(
            Some(&json!({ "id": "steadfast-sub-city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(2.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("sub rate");
    // base 100 + 1kg extra * 20 = 120
    assert_eq!(sub_rate, dec!(120.00));

    let outside_rate = provider
        .calculate_price(Some(&json!({ "id": "steadfast-outside-city" })), None, None)
        .await
        .expect("outside rate");
    assert_eq!(outside_rate, dec!(130.00));
}

#[tokio::test]
async fn steadfast_consignment_lifecycle_wiremock() {
    let mock_server = MockServer::start().await;

    // 1. Mock create_order
    Mock::given(method("POST"))
        .and(path("/create_order"))
        .and(header("Api-Key", "key_stdf_123"))
        .and(header("Secret-Key", "sec_stdf_456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 200,
            "message": "Order created successfully",
            "consignment": {
                "consignment_id": 554433,
                "invoice": "INV-STDF-99",
                "tracking_code": "STDF-TRK-554433",
                "status": "in_review"
            }
        })))
        .mount(&mock_server)
        .await;

    // 2. Mock balance
    Mock::given(method("GET"))
        .and(path("/get_balance"))
        .and(header("Api-Key", "key_stdf_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 200,
            "current_balance": "4500.50"
        })))
        .mount(&mock_server)
        .await;

    let provider = SteadfastFulfillmentProvider::new(SteadfastConfig {
        base_url: mock_server.uri(),
        api_key: "key_stdf_123".to_string(),
        secret_key: "sec_stdf_456".to_string(),
    });

    let balance = provider.get_balance().await.expect("get balance");
    assert_eq!(balance, dec!(4500.50));

    let items = vec![CreateFulfillmentItemInput {
        title: "Cotton Shirt".to_string(),
        sku: Some("SHIRT-01".to_string()),
        barcode: None,
        line_item_id: "li_shirt".to_string(),
        inventory_item_id: None,
        quantity: 1,
        metadata: None,
    }];

    let resp = provider
        .create_fulfillment(
            Some(&json!({
                "invoice": "INV-STDF-99",
                "recipient_name": "Karimullah",
                "recipient_phone": "01811111111",
                "recipient_address": "Chittagong Port",
                "cod_amount": 1500
            })),
            &items,
            None,
        )
        .await
        .expect("create fulfillment");

    let data = resp.data.expect("fulfillment data");
    assert_eq!(
        data.get("consignment_id").and_then(|v| v.as_str()),
        Some("554433")
    );
    assert_eq!(
        data.get("tracking_code").and_then(|v| v.as_str()),
        Some("STDF-TRK-554433")
    );

    assert_eq!(resp.labels.len(), 1);
    assert_eq!(resp.labels[0].tracking_number, "STDF-TRK-554433");

    // Return flow
    let ret = provider
        .create_return_fulfillment(Some(&json!({ "tracking_code": "STDF-TRK-554433" })))
        .await
        .expect("create return");
    assert_eq!(
        ret.data.unwrap().get("is_return").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[tokio::test]
async fn failed_steadfast_create_is_not_reported_as_a_fake_consignment() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/create_order"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;
    let provider = SteadfastFulfillmentProvider::new(SteadfastConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
        secret_key: "test-secret-key".to_string(),
    });

    let result = provider
        .create_fulfillment(
            Some(&json!({
                "invoice": "order-stable-123",
                "recipient_name": "Test Buyer",
                "recipient_phone": "01711111111",
                "recipient_address": "House 1, Dhaka",
                "cod_amount": 0
            })),
            &[],
            None,
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn steadfast_rate_limit_does_not_retry_shipment_creation() {
    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/create_order"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&mock_server)
        .await;
    let provider = SteadfastFulfillmentProvider::new(SteadfastConfig {
        base_url: mock_server.uri(),
        api_key: "test-api-key".to_string(),
        secret_key: "test-secret-key".to_string(),
    });

    let result = provider
        .create_fulfillment(
            Some(&json!({
                "invoice": "order-rate-limited-123",
                "recipient_name": "Test Buyer",
                "recipient_phone": "01711111111",
                "recipient_address": "House 1, Dhaka",
                "cod_amount": 0
            })),
            &[],
            None,
        )
        .await;

    assert!(result.is_err());
    assert_eq!(mock_server.received_requests().await.unwrap().len(), 1);
}
