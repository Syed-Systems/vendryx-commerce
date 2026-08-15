use std::sync::Arc;
use stheno_notification::{NotificationService, SendNotificationInput};
use stheno_notification_webpush::{VapidConfig, WebPushNotificationProvider};

#[tokio::test]
async fn test_vapid_web_push_provider() {
    let config = VapidConfig {
        subject: "mailto:admin@stheno.store".to_string(),
        public_key: "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA_0QTpQtUbVlUls0VJXg7A8u-Ts1Xbjhazmc3wh-MbZpb30u380".to_string(),
        private_key: "mock_private_key".to_string(),
    };

    let provider = Arc::new(WebPushNotificationProvider::new(config));
    let service = NotificationService::new();
    service.register_provider(provider.clone());

    // Send push notification using subscription JSON
    let sub_json = serde_json::json!({
        "endpoint": "https://fcm.googleapis.com/fcm/send/sample_token_123",
        "keys": {
            "p256dh": "BNcRdreALRFXTkOOUHK1EtK2wtaz5Ry4YfYCA",
            "auth": "tBHItJI5svbpez7KI4CCXg"
        }
    })
    .to_string();

    let notif = service
        .send_notification(SendNotificationInput {
            to: sub_json,
            channel: "push".to_string(),
            template: None,
            subject: Some("New Sale Alert!".to_string()),
            body: Some("You received an order for $120.00".to_string()),
            data: Some(serde_json::json!({ "order_id": "ord_999", "amount": "$120.00" })),
            trigger_type: Some("order.placed".to_string()),
            resource_id: Some("ord_999".to_string()),
            resource_type: Some("order".to_string()),
            receiver_id: Some("usr_admin".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(notif.status, "sent");
    assert_eq!(notif.channel, "push");

    let pushes = provider.get_dispatched_pushes();
    assert_eq!(pushes.len(), 1);
    assert_eq!(
        pushes[0].endpoint,
        "https://fcm.googleapis.com/fcm/send/sample_token_123"
    );
    assert_eq!(pushes[0].title, "New Sale Alert!");
    assert_eq!(pushes[0].body, "You received an order for $120.00");
}
