//! P23-07: Steadfast status mapping with fixtures.

use serde_json::json;
use stheno_fulfillment::{CourierStatus, FulfillmentProvider};
use stheno_fulfillment_steadfast::SteadfastFulfillmentProvider;

#[test]
fn steadfast_every_documented_status_mapped() {
    let provider = SteadfastFulfillmentProvider::default();

    let test_cases = vec![
        ("in_review", CourierStatus::Created),
        ("pending", CourierStatus::Created),
        ("picked", CourierStatus::PickedUp),
        ("collected", CourierStatus::PickedUp),
        ("in_transit", CourierStatus::InTransit),
        ("dispatched", CourierStatus::InTransit),
        ("out_for_delivery", CourierStatus::OutForDelivery),
        ("delivered", CourierStatus::Delivered),
        ("partial_delivered", CourierStatus::PartiallyDelivered),
        ("partial_delivery", CourierStatus::PartiallyDelivered),
        ("delivery_failed", CourierStatus::FailedAttempt),
        ("attempted", CourierStatus::FailedAttempt),
        ("hold", CourierStatus::OnHold),
        ("on_hold", CourierStatus::OnHold),
        ("return_in_transit", CourierStatus::ReturnInTransit),
        ("return_pending", CourierStatus::ReturnInTransit),
        ("returned", CourierStatus::Returned),
        ("returned_to_merchant", CourierStatus::Returned),
        ("cancelled", CourierStatus::Cancelled),
        ("canceled", CourierStatus::Cancelled),
        ("lost", CourierStatus::Lost),
        ("damaged", CourierStatus::Damaged),
    ];

    for (raw, expected) in test_cases {
        let payload = json!({
            "consignment_id": 987654,
            "status": raw,
            "updated_at": "2026-08-21T14:00:00Z"
        });

        let mapped = provider
            .map_webhook_event(&payload)
            .expect("map succeeds")
            .expect("mapped");

        assert_eq!(mapped.0, "987654");
        assert_eq!(mapped.1, expected, "Mismatch for status '{}'", raw);
    }
}

#[test]
fn steadfast_unmapped_status_is_unknown() {
    let provider = SteadfastFulfillmentProvider::default();

    let payload = json!({
        "consignment_id": "STDF-9999",
        "status": "custom_cyclone_warning_delay"
    });

    let (cid, status) = provider
        .map_webhook_event(&payload)
        .expect("map succeeds")
        .expect("mapped");

    assert_eq!(cid, "STDF-9999");
    assert_eq!(
        status,
        CourierStatus::Unknown("custom_cyclone_warning_delay".to_string())
    );
}
