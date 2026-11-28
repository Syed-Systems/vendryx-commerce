//! P23-05: Pathao status webhook mapping, every documented status covered by a fixture.

use serde_json::json;
use stheno_fulfillment::{CourierStatus, FulfillmentProvider};
use stheno_fulfillment_pathao::PathaoFulfillmentProvider;

#[test]
fn pathao_every_documented_status_mapped() {
    let provider = PathaoFulfillmentProvider::default();

    let test_cases = vec![
        // (raw status, expected canonical variant)
        ("Order Created", CourierStatus::Created),
        ("Pending", CourierStatus::Created),
        ("Pickup Requested", CourierStatus::Created),
        ("Pickup Pending", CourierStatus::Created),
        ("Assigned for Pickup", CourierStatus::PickedUp),
        ("Pickup in Progress", CourierStatus::PickedUp),
        ("Picked", CourierStatus::PickedUp),
        ("In Transit", CourierStatus::InTransit),
        ("Received at Hub", CourierStatus::InTransit),
        ("Dispatched", CourierStatus::InTransit),
        ("Out for Delivery", CourierStatus::OutForDelivery),
        ("Delivered", CourierStatus::Delivered),
        ("Delivery Completed", CourierStatus::Delivered),
        ("Partial Delivery", CourierStatus::PartiallyDelivered),
        ("Partially Delivered", CourierStatus::PartiallyDelivered),
        ("Delivery Failed", CourierStatus::FailedAttempt),
        ("Customer Unavailable", CourierStatus::FailedAttempt),
        ("Hold", CourierStatus::OnHold),
        ("On Hold", CourierStatus::OnHold),
        ("Delayed", CourierStatus::OnHold),
        ("Return in Progress", CourierStatus::ReturnInTransit),
        ("Return Requested", CourierStatus::ReturnInTransit),
        ("Returned", CourierStatus::Returned),
        ("Returned to Merchant", CourierStatus::Returned),
        ("Order Cancelled", CourierStatus::Cancelled),
        ("Cancelled", CourierStatus::Cancelled),
        ("Lost", CourierStatus::Lost),
        ("Damaged", CourierStatus::Damaged),
    ];

    for (raw, expected) in test_cases {
        let payload = json!({
            "consignment_id": "PT-TRACK-1001",
            "merchant_order_id": "ORD-500",
            "order_status": raw,
            "updated_at": "2026-08-21T12:00:00Z"
        });

        let mapped = provider
            .map_webhook_event(&payload)
            .expect("map webhook event")
            .expect("payload successfully mapped");

        assert_eq!(mapped.0, "PT-TRACK-1001");
        assert_eq!(mapped.1, expected, "Mismatch for status '{raw}'");
    }
}

#[test]
fn pathao_webhook_slug_and_unmapped_status() {
    let provider = PathaoFulfillmentProvider::default();

    // Mapping using order_status_slug
    let payload_slug = json!({
        "consignment_id": "PT-TRACK-1002",
        "order_status_slug": "delivered"
    });
    let (cid, status) = provider
        .map_webhook_event(&payload_slug)
        .expect("map succeeds")
        .expect("mapped");
    assert_eq!(cid, "PT-TRACK-1002");
    assert_eq!(status, CourierStatus::Delivered);

    // Unmapped status preserved as Unknown
    let payload_unknown = json!({
        "consignment_id": "PT-TRACK-9999",
        "order_status": "custom_pathao_flood_emergency"
    });
    let (cid, status_unknown) = provider
        .map_webhook_event(&payload_unknown)
        .expect("map succeeds")
        .expect("mapped");
    assert_eq!(cid, "PT-TRACK-9999");
    assert_eq!(
        status_unknown,
        CourierStatus::Unknown("custom_pathao_flood_emergency".to_string())
    );
}
