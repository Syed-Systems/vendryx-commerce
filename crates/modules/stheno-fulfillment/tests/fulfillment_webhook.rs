//! P21-04: FulfillmentService::receive_webhook with its own replay guard and event table.

use serde_json::json;
use stheno_db::testing::TestDb;
use stheno_db::{create_pool, tenant};
use stheno_fulfillment::FulfillmentService;

const TENANT: &str = TestDb::TEST_TENANT;

#[tokio::test]
async fn fulfillment_webhook_records_event_and_ignores_replays() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = FulfillmentService::new(pool);

    let provider_id = "fp_steadfast";
    let raw_payload_1 = serde_json::to_vec(&json!({
        "consignment_id": "cons_track_12345",
        "status": "delivered",
        "city": "Dhaka"
    }))
    .expect("json payload");

    // First webhook delivery -> received
    let event1 = tenant::scope(TENANT.to_string(), {
        let service = service.clone();
        let payload = raw_payload_1.clone();
        async move {
            service
                .receive_webhook(provider_id, &payload, None)
                .await
                .expect("receive webhook succeeds")
        }
    })
    .await;

    assert_eq!(event1.provider_id, provider_id);
    assert_eq!(event1.provider_event_id, "cons_track_12345");
    assert_eq!(event1.state, "received");

    // Replay of the exact same event -> ignored
    let event2 = tenant::scope(TENANT.to_string(), {
        let service = service.clone();
        let payload = raw_payload_1.clone();
        async move {
            service
                .receive_webhook(provider_id, &payload, None)
                .await
                .expect("replay succeeds")
        }
    })
    .await;

    assert_eq!(event2.id, event1.id);
    assert_eq!(event2.state, "ignored", "replayed webhook must be ignored");

    // Different event id -> received
    let raw_payload_2 = serde_json::to_vec(&json!({
        "consignment_id": "cons_track_99999",
        "status": "in_transit",
        "city": "Chittagong"
    }))
    .expect("json payload 2");

    let event3 = tenant::scope(TENANT.to_string(), {
        let service = service.clone();
        async move {
            service
                .receive_webhook(provider_id, &raw_payload_2, None)
                .await
                .expect("receive second webhook succeeds")
        }
    })
    .await;

    assert_eq!(event3.provider_event_id, "cons_track_99999");
    assert_eq!(event3.state, "received");
    assert_ne!(event3.id, event1.id);

    db.drop().await;
}
