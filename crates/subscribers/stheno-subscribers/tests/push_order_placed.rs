use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_events::Event;
use stheno_notification::NotificationService;
use stheno_notification_webpush::{VapidConfig, WebPushNotificationProvider};
use stheno_subscribers::{EventSubscriber, PushOrderPlacedSubscriber};

#[tokio::test]
async fn test_push_order_placed_subscriber_social_suppression() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let vapid_config = VapidConfig {
        subject: "mailto:admin@stheno.store".to_string(),
        public_key: "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA".to_string(),
        private_key: "mock_key".to_string(),
    };

    let push_provider = Arc::new(WebPushNotificationProvider::new(vapid_config));
    let notif_service = Arc::new(NotificationService::new(pool));
    notif_service.register_provider(push_provider.clone());

    let subscriber = PushOrderPlacedSubscriber::new(
        notif_service,
        "https://fcm.googleapis.com/fcm/send/owner_device_token",
    );

    // 1. Regular storefront order -> Web Push MUST be sent
    let storefront_event = Event::new(
        "order.placed",
        serde_json::json!({
            "order_id": "ord_storefront_001",
            "total_amount": "$75.00",
            "metadata": {
                "source": "storefront"
            }
        }),
    );

    subscriber.handle(&storefront_event).await.unwrap();

    let sent_pushes = push_provider.get_dispatched_pushes();
    assert_eq!(sent_pushes.len(), 1);
    assert_eq!(
        sent_pushes[0].title,
        "New Order Placed: #ord_storefront_001"
    );
    assert!(sent_pushes[0].body.contains("$75.00"));

    // 2. Social order (metadata.source == "social") -> Web Push MUST BE SKIPPED (PRODUCT §5.5 trap 3)
    let social_event = Event::new(
        "order.placed",
        serde_json::json!({
            "order_id": "ord_social_draft_002",
            "total_amount": "$150.00",
            "metadata": {
                "source": "social"
            }
        }),
    );

    subscriber.handle(&social_event).await.unwrap();

    // Dispatched count must still be 1 (no new push dispatched)
    let updated_pushes = push_provider.get_dispatched_pushes();
    assert_eq!(updated_pushes.len(), 1);

    // 3. Social order with top-level source field -> Web Push MUST ALSO BE SKIPPED
    let social_event_top_level = Event::new(
        "order.placed",
        serde_json::json!({
            "order_id": "ord_social_003",
            "total_amount": "$200.00",
            "source": "social"
        }),
    );

    subscriber.handle(&social_event_top_level).await.unwrap();
    let final_pushes = push_provider.get_dispatched_pushes();
    assert_eq!(final_pushes.len(), 1);
}
