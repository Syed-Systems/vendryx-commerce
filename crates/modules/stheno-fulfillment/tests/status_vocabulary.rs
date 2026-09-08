//! P23-01: Courier status vocabulary: one engine-side enum, plus an explicit Unknown(String) that logs and alerts.

use stheno_fulfillment::CourierStatus;

#[test]
fn canonical_status_parsing() {
    assert_eq!(CourierStatus::parse("created"), CourierStatus::Created);
    assert_eq!(CourierStatus::parse("pending"), CourierStatus::Created);
    assert_eq!(CourierStatus::parse("order_placed"), CourierStatus::Created);

    assert_eq!(CourierStatus::parse("picked_up"), CourierStatus::PickedUp);
    assert_eq!(CourierStatus::parse("pickup"), CourierStatus::PickedUp);
    assert_eq!(CourierStatus::parse("collected"), CourierStatus::PickedUp);

    assert_eq!(CourierStatus::parse("in_transit"), CourierStatus::InTransit);
    assert_eq!(CourierStatus::parse("dispatched"), CourierStatus::InTransit);
    assert_eq!(CourierStatus::parse("transit"), CourierStatus::InTransit);

    assert_eq!(
        CourierStatus::parse("out_for_delivery"),
        CourierStatus::OutForDelivery
    );
    assert_eq!(
        CourierStatus::parse("delivery_in_progress"),
        CourierStatus::OutForDelivery
    );

    assert_eq!(CourierStatus::parse("delivered"), CourierStatus::Delivered);
    assert_eq!(CourierStatus::parse("completed"), CourierStatus::Delivered);

    assert_eq!(
        CourierStatus::parse("partially_delivered"),
        CourierStatus::PartiallyDelivered
    );
    assert_eq!(
        CourierStatus::parse("partial_delivery"),
        CourierStatus::PartiallyDelivered
    );

    assert_eq!(
        CourierStatus::parse("failed_attempt"),
        CourierStatus::FailedAttempt
    );
    assert_eq!(
        CourierStatus::parse("delivery_failed"),
        CourierStatus::FailedAttempt
    );

    assert_eq!(CourierStatus::parse("on_hold"), CourierStatus::OnHold);
    assert_eq!(CourierStatus::parse("hold"), CourierStatus::OnHold);

    assert_eq!(
        CourierStatus::parse("return_in_transit"),
        CourierStatus::ReturnInTransit
    );
    assert_eq!(
        CourierStatus::parse("returning"),
        CourierStatus::ReturnInTransit
    );

    assert_eq!(CourierStatus::parse("returned"), CourierStatus::Returned);
    assert_eq!(CourierStatus::parse("return"), CourierStatus::Returned);

    assert_eq!(CourierStatus::parse("cancelled"), CourierStatus::Cancelled);
    assert_eq!(CourierStatus::parse("canceled"), CourierStatus::Cancelled);

    assert_eq!(CourierStatus::parse("lost"), CourierStatus::Lost);
    assert_eq!(CourierStatus::parse("damaged"), CourierStatus::Damaged);
}

#[test]
fn normalization_tolerates_case_and_dashes() {
    assert_eq!(
        CourierStatus::parse("OUT-FOR-DELIVERY"),
        CourierStatus::OutForDelivery
    );
    assert_eq!(CourierStatus::parse("In Transit"), CourierStatus::InTransit);
    assert_eq!(CourierStatus::parse("Delivered "), CourierStatus::Delivered);
    assert_eq!(
        CourierStatus::parse("Partially Delivered"),
        CourierStatus::PartiallyDelivered
    );
}

#[test]
fn unknown_status_captured_in_unknown_variant() {
    let raw = "custom_carrier_warehouse_exception";
    let status = CourierStatus::parse(raw);

    assert!(status.is_unknown());
    assert_eq!(status, CourierStatus::Unknown(raw.to_string()));
    assert_eq!(status.unknown_raw(), Some(raw));
    assert_eq!(status.as_str(), raw);
}

#[test]
fn status_predicates() {
    let delivered = CourierStatus::Delivered;
    assert!(delivered.is_delivered());
    assert!(delivered.is_terminal());
    assert!(!delivered.is_return());
    assert!(!delivered.is_unknown());

    let returning = CourierStatus::ReturnInTransit;
    assert!(!returning.is_delivered());
    assert!(!returning.is_terminal());
    assert!(returning.is_return());

    let returned = CourierStatus::Returned;
    assert!(returned.is_terminal());
    assert!(returned.is_return());

    let cancelled = CourierStatus::Cancelled;
    assert!(cancelled.is_terminal());
    assert!(!cancelled.is_return());

    let unknown = CourierStatus::Unknown("custom_state".to_string());
    assert!(!unknown.is_delivered());
    assert!(!unknown.is_terminal());
    assert!(!unknown.is_return());
    assert!(unknown.is_unknown());
}

#[test]
fn serde_roundtrip() {
    let status = CourierStatus::OutForDelivery;
    let json = serde_json::to_string(&status).expect("serialize");
    assert_eq!(json, "\"out_for_delivery\"");
    let parsed: CourierStatus = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed, CourierStatus::OutForDelivery);

    let unknown = CourierStatus::Unknown("alien_carrier_state".to_string());
    let json_unknown = serde_json::to_string(&unknown).expect("serialize unknown");
    assert_eq!(json_unknown, "\"alien_carrier_state\"");
    let parsed_unknown: CourierStatus =
        serde_json::from_str(&json_unknown).expect("deserialize unknown");
    assert_eq!(
        parsed_unknown,
        CourierStatus::Unknown("alien_carrier_state".to_string())
    );
}
