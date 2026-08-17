use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_notification::{NotificationService, SendNotificationInput};
use stheno_notification_sendgrid::{SendGridConfig, SendGridNotificationProvider};

#[tokio::test]
async fn test_sendgrid_notification_provider() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let config = SendGridConfig {
        api_key: "SG.dummy_api_key_test".to_string(),
        from_email: "orders@sthenostore.com".to_string(),
        from_name: Some("Stheno Orders".to_string()),
    };

    let provider = Arc::new(SendGridNotificationProvider::new(config));
    let service = NotificationService::new(pool);
    service.register_provider(provider.clone());

    let notif = service
        .send_notification(SendNotificationInput {
            to: "bob@example.com".to_string(),
            channel: "email".to_string(),
            template: None,
            subject: Some("Invoice #5432".to_string()),
            body: Some("Your invoice is attached.".to_string()),
            data: None,
            trigger_type: Some("invoice.created".to_string()),
            resource_id: Some("inv_5432".to_string()),
            resource_type: Some("invoice".to_string()),
            receiver_id: Some("cus_bob".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(notif.status, "sent");
    assert_eq!(notif.to, "bob@example.com");

    let records = provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "bob@example.com");
    assert_eq!(records[0].from_email, "orders@sthenostore.com");
    assert_eq!(records[0].subject, "Invoice #5432");
}
