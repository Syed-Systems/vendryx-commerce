use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use stheno_db::create_pool;
use stheno_db::diesel::sql_types::BigInt;
use stheno_db::diesel::{self, QueryableByName};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::testing::TestDb;
use stheno_notification::{
    CreateTemplateInput, NotificationProvider, NotificationProviderResult, NotificationService,
    SendNotificationInput,
};
use stheno_types::Result;

type SentNotificationRecord = (String, String, Option<String>, String);

#[derive(QueryableByName)]
struct ConsentCount {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

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
    assert!(service
        .has_sent_for_resource("order.placed", "order", "ord_1001")
        .await
        .unwrap());
    assert!(!service
        .has_sent_for_resource("order.placed", "order", "ord_other")
        .await
        .unwrap());

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

#[tokio::test]
async fn anonymous_whatsapp_analytics_consent_is_separate_and_revokeable_by_customer() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = NotificationService::new(pool.clone());

    stheno_db::tenant::scope("ten_whatsapp_analytics", async {
        service
            .record_whatsapp_analytics_consent(
                "ten_whatsapp_analytics",
                "+8801712345678",
                Some("cus_analytics_1"),
                "ord_analytics_1",
            )
            .await
            .expect("record explicit analytics consent");

        let counts = stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT count(*)::bigint AS count FROM notification.whatsapp_consent \
                     WHERE customer_id = 'cus_analytics_1' AND consented = true",
                )
                .get_result::<ConsentCount>(&mut conn)
                .await
                .map(|row| row.count)
                .map_err(|error| stheno_types::Error::Database(error.to_string()))
            })
        })
        .await
        .expect("read messaging opt-in ledger");
        assert_eq!(
            counts, 0,
            "analytics consent must not opt into notifications"
        );

        assert!(service
            .revoke_whatsapp_consent_for_customer("ten_whatsapp_analytics", "cus_analytics_1")
            .await
            .expect("withdraw customer messaging and analytics consent"));
        let analytics = stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT count(*)::bigint AS count FROM notification.whatsapp_analytics_consent \
                     WHERE customer_id = 'cus_analytics_1' AND consented = true",
                )
                .get_result::<ConsentCount>(&mut conn)
                .await
                .map(|row| row.count)
                .map_err(|error| stheno_types::Error::Database(error.to_string()))
            })
        })
        .await
        .expect("read analytics consent ledger");
        assert_eq!(analytics, 0);

        service
            .record_whatsapp_analytics_consent(
                "ten_whatsapp_analytics",
                "+8801712345678",
                Some("cus_analytics_1"),
                "ord_analytics_1",
            )
            .await
            .expect("replay original checkout consent idempotently");
        let after_replay = stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT count(*)::bigint AS count FROM notification.whatsapp_analytics_consent \
                     WHERE customer_id = 'cus_analytics_1' AND consented = true",
                )
                .get_result::<ConsentCount>(&mut conn)
                .await
                .map(|row| row.count)
                .map_err(|error| stheno_types::Error::Database(error.to_string()))
            })
        })
        .await
        .expect("read analytics consent after replay");
        assert_eq!(
            after_replay, 0,
            "a checkout replay must not undo revocation"
        );

        service
            .record_whatsapp_analytics_consent(
                "ten_whatsapp_analytics",
                "+8801712345678",
                Some("cus_analytics_1"),
                "ord_analytics_2",
            )
            .await
            .expect("record a new explicit opt-in");
        let after_new_opt_in = stheno_db::tenant::with_tenant_conn(&pool, |mut conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT count(*)::bigint AS count FROM notification.whatsapp_analytics_consent \
                     WHERE customer_id = 'cus_analytics_1' AND consented = true",
                )
                .get_result::<ConsentCount>(&mut conn)
                .await
                .map(|row| row.count)
                .map_err(|error| stheno_types::Error::Database(error.to_string()))
            })
        })
        .await
        .expect("read analytics consent after new opt-in");
        assert_eq!(after_new_opt_in, 1);
    })
    .await;
    db.drop().await;
}

#[tokio::test]
async fn notification_outbox_is_durable_and_idempotent() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");
    let service = NotificationService::new(pool);
    let sent_log = Arc::new(RwLock::new(Vec::new()));
    service.register_provider(Arc::new(MockNotificationProvider {
        sent: sent_log.clone(),
    }));
    let input = || SendNotificationInput {
        to: "owner@example.com".to_string(),
        channel: "email".to_string(),
        template: None,
        subject: Some("Trial reminder".to_string()),
        body: Some("Your trial ends soon".to_string()),
        data: None,
        trigger_type: Some("trial_warning_day-3".to_string()),
        resource_id: Some("ten_store".to_string()),
        resource_type: Some("trial_subscription".to_string()),
        receiver_id: None,
    };

    assert!(service
        .enqueue_notification_idempotent(input(), "trial-warning/store/day-3")
        .await
        .unwrap());
    assert!(!service
        .enqueue_notification_idempotent(input(), "trial-warning/store/day-3")
        .await
        .unwrap());
    assert!(sent_log.read().unwrap().is_empty());

    assert_eq!(service.deliver_due_outbox(10).await.unwrap(), (1, 0));
    assert_eq!(sent_log.read().unwrap().len(), 1);
    assert!(service
        .has_sent_for_resource("trial_warning_day-3", "trial_subscription", "ten_store")
        .await
        .unwrap());
    db.drop().await;
}
