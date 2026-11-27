//! Analytics events survive the service instance that wrote them.

use stheno_analytics::{AnalyticsService, TrackEventInput};
use stheno_db::create_pool;
use stheno_db::testing::TestDb;

#[tokio::test]
async fn rows_outlive_the_service_instance() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let writer = AnalyticsService::new(pool.clone());
    writer
        .track_event(TrackEventInput {
            event_name: "order.completed".to_string(),
            distinct_id: "cus_outlive".to_string(),
            properties: serde_json::json!({ "order_id": "ord_1" }),
        })
        .await
        .expect("track");
    drop(writer);

    let reader = AnalyticsService::new(pool);
    let events = reader
        .list_events_for_distinct_id("cus_outlive")
        .await
        .expect("list");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_name, "order.completed");
    assert_eq!(events[0].distinct_id, "cus_outlive");

    db.drop().await;
}
