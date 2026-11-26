use std::sync::Arc;
use stheno_db::create_pool;
use stheno_db::testing::TestDb;
use stheno_events::Event;
use stheno_notification::NotificationService;
use stheno_notification_resend::{ResendConfig, ResendNotificationProvider};
use stheno_subscribers::{
    EventSubscriber, IdentityCreatedSubscriber, OrderConfirmationEmailSubscriber,
    PasswordResetSubscriber, UserInviteSubscriber,
};

#[tokio::test]
async fn test_identity_created_subscriber_sends_welcome_email() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let resend_config = ResendConfig {
        api_key: "".to_string(),
        from_email: "notifications@vendryx.store".to_string(),
        from_name: Some("Vendryx".to_string()),
        base_url: None,
    };
    let resend_provider = Arc::new(ResendNotificationProvider::new(resend_config));

    let notif_service = Arc::new(NotificationService::new(pool));
    notif_service.register_provider(resend_provider.clone());

    let subscriber = IdentityCreatedSubscriber::new(notif_service, "Vendryx");

    // 1. Storefront Customer registration
    let customer_event = Event::new(
        "auth.identity_created",
        serde_json::json!({
            "identity_id": "ident_cust_001",
            "provider_id": "emailpass",
            "entity_id": "customer@example.com",
            "actor_type": "customer",
            "actor_id": "cust_001"
        }),
    );

    subscriber
        .handle(&customer_event)
        .await
        .expect("handle customer registration");

    let records = resend_provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "customer@example.com");
    assert!(records[0].subject.contains("Welcome to Vendryx"));
    assert!(records[0].body.contains("customer@example.com"));
    assert!(records[0].body.contains("View Your Account"));

    // 2. Merchant Admin User creation
    let user_event = Event::new(
        "auth.identity_created",
        serde_json::json!({
            "identity_id": "ident_user_002",
            "provider_id": "emailpass",
            "entity_id": "merchant@vendryx.store",
            "actor_type": "user",
            "actor_id": "usr_002"
        }),
    );

    subscriber
        .handle(&user_event)
        .await
        .expect("handle merchant user creation");

    let records = resend_provider.get_sent_records();
    assert_eq!(records.len(), 2);
    assert_eq!(records[1].to, "merchant@vendryx.store");
    assert!(records[1].subject.contains("Merchant Workspace"));
    assert!(records[1].body.contains("merchant@vendryx.store"));
    assert!(records[1].body.contains("Sign In to Dashboard"));
}

#[tokio::test]
async fn test_password_reset_subscriber_sends_recovery_email() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let resend_config = ResendConfig {
        api_key: "".to_string(),
        from_email: "security@vendryx.store".to_string(),
        from_name: Some("Vendryx Security".to_string()),
        base_url: None,
    };
    let resend_provider = Arc::new(ResendNotificationProvider::new(resend_config));

    let notif_service = Arc::new(NotificationService::new(pool));
    notif_service.register_provider(resend_provider.clone());

    let subscriber = PasswordResetSubscriber::new(notif_service, "Vendryx");

    let reset_event = Event::new(
        "auth.password_reset_requested",
        serde_json::json!({
            "identity_id": "ident_reset_001",
            "entity_id": "user@example.com",
            "reset_token": "a1b2c3d4e5f6",
            "actor_type": "customer"
        }),
    );

    subscriber
        .handle(&reset_event)
        .await
        .expect("handle password reset");

    let records = resend_provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "user@example.com");
    assert!(records[0].subject.contains("Reset your Vendryx password"));
    assert!(records[0].body.contains("a1b2c3d4e5f6"));
}

#[tokio::test]
async fn test_user_invite_subscriber_sends_invitation_email() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let resend_config = ResendConfig {
        api_key: "".to_string(),
        from_email: "notifications@vendryx.store".to_string(),
        from_name: Some("Vendryx".to_string()),
        base_url: None,
    };
    let resend_provider = Arc::new(ResendNotificationProvider::new(resend_config));

    let notif_service = Arc::new(NotificationService::new(pool));
    notif_service.register_provider(resend_provider.clone());

    let subscriber = UserInviteSubscriber::new(notif_service, "Vendryx");

    let invite_event = Event::new(
        "user.invite_created",
        serde_json::json!({
            "invite_id": "inv_001",
            "email": "teammate@example.com",
            "role": "catalog_manager",
            "token": "tok_invite_999",
            "expires_at": "2026-09-16T00:00:00Z"
        }),
    );

    subscriber
        .handle(&invite_event)
        .await
        .expect("handle invite");

    let records = resend_provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "teammate@example.com");
    assert!(records[0].subject.contains("invited to join Vendryx Admin"));
    assert!(records[0].body.contains("catalog_manager"));
    assert!(records[0].body.contains("tok_invite_999"));
}

#[tokio::test]
async fn test_order_confirmation_email_subscriber_sends_email() {
    let db = TestDb::migrated().await;
    let pool = create_pool(&db.url, 5).expect("pool");

    let resend_config = ResendConfig {
        api_key: "".to_string(),
        from_email: "orders@vendryx.store".to_string(),
        from_name: Some("Vendryx Orders".to_string()),
        base_url: None,
    };
    let resend_provider = Arc::new(ResendNotificationProvider::new(resend_config));

    let notif_service = Arc::new(NotificationService::new(pool));
    notif_service.register_provider(resend_provider.clone());

    let subscriber = OrderConfirmationEmailSubscriber::new(notif_service.clone(), "Vendryx");

    let order_event = Event::new(
        "order.placed",
        serde_json::json!({
            "order_id": "ord_vx_1001",
            "email": "buyer@example.com",
            "total_amount": "$125.00"
        }),
    );

    subscriber
        .handle(&order_event)
        .await
        .expect("handle order confirmation");

    let (delivered, failed) = notif_service
        .deliver_due_outbox(1)
        .await
        .expect("deliver queued order confirmation");
    assert_eq!(delivered, 1);
    assert_eq!(failed, 0);

    let records = resend_provider.get_sent_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].to, "buyer@example.com");
    assert!(
        records[0]
            .subject
            .contains("Order Confirmation #ord_vx_1001")
    );
    assert!(records[0].body.contains("$125.00"));
}
