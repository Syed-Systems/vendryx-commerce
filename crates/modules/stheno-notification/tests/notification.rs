use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_notification::{
    CreateTemplateInput, NotificationProvider, NotificationProviderResult, NotificationService,
    SendNotificationInput,
};
use stheno_types::Result;

type SentNotificationRecord = (String, String, Option<String>, String);

struct MockNotificationProvider {
    sent: Arc<RwLock<Vec<SentNotificationRecord>>>,
}

#[async_trait]
impl NotificationProvider for MockNotificationProvider {
    fn provider_id(&self) -> &'static str {
        "mock"
    }

    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["email", "push"]
    }

    async fn send(
        &self,
        to: &str,
        channel: &str,
        subject: Option<&str>,
        body: &str,
        _data: Option<&serde_json::Value>,
    ) -> Result<NotificationProviderResult> {
        self.sent.write().unwrap().push((
            to.to_string(),
            channel.to_string(),
            subject.map(|s| s.to_string()),
            body.to_string(),
        ));
        Ok(NotificationProviderResult {
            provider_message_id: Some("mock_123".to_string()),
            metadata: None,
        })
    }
}

#[tokio::test]
async fn test_notification_template_rendering_and_routing() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = NotificationService::new(pool);
    let sent_log = Arc::new(RwLock::new(Vec::new()));
    let provider = Arc::new(MockNotificationProvider {
        sent: sent_log.clone(),
    });

    service.register_provider(provider);

    // 1. Create order placed template
    let template = service
        .create_template(CreateTemplateInput {
            name: "order_placed".to_string(),
            channel: "email".to_string(),
            subject_template: Some("Order #{{order_id}} Confirmed".to_string()),
            body_template:
                "Hi {{customer_name}}, thank you for your order #{{order_id}} of {{amount}}!"
                    .to_string(),
        })
        .await
        .unwrap();

    assert_eq!(template.name, "order_placed");

    // 2. Send notification via template
    let notif = service
        .send_notification(SendNotificationInput {
            to: "customer@example.com".to_string(),
            channel: "email".to_string(),
            template: Some("order_placed".to_string()),
            subject: None,
            body: None,
            data: Some(serde_json::json!({
                "order_id": "ord_1001",
                "customer_name": "Alice",
                "amount": "$49.99"
            })),
            trigger_type: Some("order.placed".to_string()),
            resource_id: Some("ord_1001".to_string()),
            resource_type: Some("order".to_string()),
            receiver_id: Some("cus_123".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(notif.status, "sent");
    assert_eq!(notif.to, "customer@example.com");

    // 3. Verify delivery record
    let deliveries = service.list_deliveries(&notif.id).await.unwrap();
    assert_eq!(deliveries.len(), 1);
    assert!(deliveries[0].delivered_at.is_some());

    // 4. Verify sent log
    {
        let sent = sent_log.read().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].2.as_deref(), Some("Order #ord_1001 Confirmed"));
        assert_eq!(
            sent[0].3,
            "Hi Alice, thank you for your order #ord_1001 of $49.99!"
        );
    }

    db.drop().await;
}
