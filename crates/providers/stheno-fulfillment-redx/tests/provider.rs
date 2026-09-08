//! P23-08: stheno-fulfillment-redx crate: options, rates, consignment, cancel, return.

use rust_decimal_macros::dec;
use serde_json::json;
use stheno_fulfillment::{CalculatePriceContext, CreateFulfillmentItemInput, FulfillmentProvider};
use stheno_fulfillment_redx::{RedxConfig, RedxFulfillmentProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn redx_options_and_zone_rates() {
    let provider = RedxFulfillmentProvider::default();
    assert_eq!(provider.identifier(), "redx");

    let options = provider.get_fulfillment_options().await.expect("options");
    assert_eq!(options.len(), 4);
    assert!(options.iter().any(|o| o.id == "redx-standard"));
    assert!(options.iter().any(|o| o.id == "redx-return" && o.is_return));

    let inside = provider
        .calculate_price(
            Some(&json!({ "tier": "inside_city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 1,
                total_weight: Some(dec!(0.8)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("inside rate");
    assert_eq!(inside, dec!(60.00));

    let sub = provider
        .calculate_price(
            Some(&json!({ "id": "redx-sub-city" })),
            None,
            None,
        )
        .await
        .expect("sub rate");
    assert_eq!(sub, dec!(100.00));

    let outside = provider
        .calculate_price(
            Some(&json!({ "id": "redx-outside-city" })),
            None,
            Some(&CalculatePriceContext {
                items_count: 2,
                total_weight: Some(dec!(3.0)),
                currency_code: "bdt".to_string(),
            }),
        )
        .await
        .expect("outside rate");
    // 130 base + 2kg extra * 25 = 180
    assert_eq!(outside, dec!(180.00));
}

#[tokio::test]
async fn redx_parcel_lifecycle_wiremock() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/parcel"))
        .and(header("API-ACCESS-TOKEN", "Bearer jwt_test_redx"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tracking_id": "REDX-TRK-7711",
            "parcel_id": 7711,
            "message": "Parcel created successfully"
        })))
        .mount(&mock_server)
        .await;

    let provider = RedxFulfillmentProvider::new(RedxConfig {
        base_url: mock_server.uri(),
        access_token: "jwt_test_redx".to_string(),
    });

    let items = vec![CreateFulfillmentItemInput {
        title: "Leather Wallet".to_string(),
        sku: Some("WALLET-01".to_string()),
        barcode: None,
        line_item_id: "li_wallet".to_string(),
        inventory_item_id: None,
        quantity: 1,
        metadata: None,
    }];

    let resp = provider
        .create_fulfillment(
            Some(&json!({
                "merchant_invoice_id": "INV-RDX-12",
                "customer_name": "Tariqul Islam",
                "customer_phone": "01911111111",
                "customer_address": "Uttara Sector 3",
                "delivery_area": "Uttara",
                "cash_collection_amount": 850
            })),
            &items,
            None,
        )
        .await
        .expect("create parcel");

    let data = resp.data.expect("data");
    assert_eq!(
        data.get("tracking_id").and_then(|v| v.as_str()),
        Some("REDX-TRK-7711")
    );
    assert_eq!(
        data.get("parcel_id").and_then(|v| v.as_str()),
        Some("7711")
    );
    assert_eq!(resp.labels.len(), 1);
    assert_eq!(resp.labels[0].tracking_number, "REDX-TRK-7711");

    // Return flow
    let ret = provider
        .create_return_fulfillment(Some(&json!({ "tracking_id": "REDX-TRK-7711" })))
        .await
        .expect("create return");
    assert!(ret.data.unwrap().get("is_return").and_then(|v| v.as_bool()).unwrap());
}
