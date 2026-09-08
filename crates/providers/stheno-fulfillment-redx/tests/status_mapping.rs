//! P23-09: RedX status mapping with fixtures.

use serde_json::json;
use stheno_fulfillment::{CourierStatus, FulfillmentProvider};
use stheno_fulfillment_redx::RedxFulfillmentProvider;

#[test]
fn redx_every_documented_status_mapped() {
    let provider = RedxFulfillmentProvider::default();

    let test_cases = vec![
        ("ready-for-pickup", CourierStatus::Created),
        ("pickup-pending", CourierStatus::Created),
        ("pending", CourierStatus::Created),
        ("picked-up", CourierStatus::PickedUp),
        ("picked", CourierStatus::PickedUp),
        ("in-transit", CourierStatus::InTransit),
        ("hub-received", CourierStatus::InTransit),
        ("dispatched", CourierStatus::InTransit),
        ("delivery-in-progress", CourierStatus::OutForDelivery),
        ("out-for-delivery", CourierStatus::OutForDelivery),
        ("delivered", CourierStatus::Delivered),
        ("partially-delivered", CourierStatus::PartiallyDelivered),
        ("partial-delivery", CourierStatus::PartiallyDelivered),
        ("delivery-failed", CourierStatus::FailedAttempt),
        ("failed-attempt", CourierStatus::FailedAttempt),
        ("hold", CourierStatus::OnHold),
        ("on-hold", CourierStatus::OnHold),
        ("returning", CourierStatus::ReturnInTransit),
        ("return-in-progress", CourierStatus::ReturnInTransit),
        ("returned", CourierStatus::Returned),
        ("cancelled", CourierStatus::Cancelled),
        ("canceled", CourierStatus::Cancelled),
        ("lost", CourierStatus::Lost),
        ("damaged", CourierStatus::Damaged),
    ];

    for (raw, expected) in test_cases {
        let payload = json!({
            "tracking_id": "REDX-TRK-8822",
            "status": raw,
            "message": "Status updated"
        });

        let mapped = provider
            .map_webhook_event(&payload)
            .expect("map succeeds")
            .expect("mapped");

        assert_eq!(mapped.0, "REDX-TRK-8822");
        assert_eq!(mapped.1, expected, "Mismatch for status '{}'", raw);
    }
}

#[test]
fn redx_unmapped_status_is_unknown() {
    let provider = RedxFulfillmentProvider::default();

    let payload = json!({
        "tracking_id": "REDX-TRK-9999",
        "status": "custom-bridge-closed-reroute"
    });

    let (tid, status) = provider
        .map_webhook_event(&payload)
        .expect("map succeeds")
        .expect("mapped");

    assert_eq!(tid, "REDX-TRK-9999");
    assert_eq!(
        status,
        CourierStatus::Unknown("custom-bridge-closed-reroute".to_string())
    );
}
