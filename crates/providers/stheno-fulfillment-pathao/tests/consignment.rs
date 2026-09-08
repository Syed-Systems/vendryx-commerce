//! P23-04: Pathao create_fulfillment returning consignment/tracking id; cancel and return flows.

use serde_json::json;
use stheno_fulfillment::{CreateFulfillmentItemInput, FulfillmentProvider};
use stheno_fulfillment_pathao::{PathaoConfig, PathaoFulfillmentProvider};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn pathao_consignment_create_cancel_return_wiremock() {
    let mock_server = MockServer::start().await;

    // 1. Mock token
    Mock::given(method("POST"))
        .and(path("/aladdin/api/v1/issue-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "token_pathao_cns",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    // 2. Mock order create
    Mock::given(method("POST"))
        .and(path("/aladdin/api/v1/orders"))
        .and(header("Authorization", "Bearer token_pathao_cns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "success",
            "code": 200,
            "message": "Order created successfully",
            "data": {
                "consignment_id": "PT-CNS-789012",
                "merchant_order_id": "ORD-BD-101",
                "order_status": "Order Created",
                "delivery_fee": 60
            }
        })))
        .mount(&mock_server)
        .await;

    // 3. Mock cancel
    Mock::given(method("POST"))
        .and(path("/aladdin/api/v1/orders/PT-CNS-789012/cancel"))
        .and(header("Authorization", "Bearer token_pathao_cns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "success",
            "code": 200,
            "message": "Order cancelled successfully"
        })))
        .mount(&mock_server)
        .await;

    let provider = PathaoFulfillmentProvider::new(PathaoConfig {
        base_url: mock_server.uri(),
        ..PathaoConfig::default()
    });

    let items = vec![CreateFulfillmentItemInput {
        title: "Kashmiri Shawl".to_string(),
        sku: Some("SHAWL-01".to_string()),
        barcode: None,
        line_item_id: "li_shawl".to_string(),
        inventory_item_id: None,
        quantity: 1,
        metadata: None,
    }];

    // Create consignment
    let resp = provider
        .create_fulfillment(
            Some(&json!({
                "merchant_order_id": "ORD-BD-101",
                "recipient_name": "Rahim Ahmed",
                "recipient_phone": "01711111111",
                "recipient_address": "House 10, Road 4, Dhanmondi",
                "amount_to_collect": 1200
            })),
            &items,
            None,
        )
        .await
        .expect("create fulfillment in pathao");

    let data = resp.data.expect("fulfillment data present");
    assert_eq!(
        data.get("consignment_id").and_then(|v| v.as_str()),
        Some("PT-CNS-789012")
    );
    assert_eq!(
        data.get("provider").and_then(|v| v.as_str()),
        Some("pathao")
    );

    assert_eq!(resp.labels.len(), 1);
    let label = &resp.labels[0];
    assert_eq!(label.tracking_number, "PT-CNS-789012");
    assert!(label
        .tracking_url
        .as_ref()
        .unwrap()
        .contains("PT-CNS-789012"));

    // Cancel flow
    provider
        .cancel_fulfillment(Some(&json!({ "consignment_id": "PT-CNS-789012" })))
        .await
        .expect("cancel fulfillment in pathao");

    // Return flow
    let return_resp = provider
        .create_return_fulfillment(Some(&json!({ "consignment_id": "PT-CNS-789012" })))
        .await
        .expect("create return fulfillment in pathao");

    let ret_data = return_resp.data.expect("return data present");
    assert_eq!(
        ret_data.get("is_return").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(return_resp.labels[0]
        .tracking_number
        .contains("PT-CNS-789012"));
}
