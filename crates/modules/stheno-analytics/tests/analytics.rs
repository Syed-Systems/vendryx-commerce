use std::sync::Arc;
use stheno_analytics::{
    AnalyticsService, IdentifyUserInput, LocalAnalyticsProvider, PostHogAnalyticsProvider,
    PostHogConfig, TrackEventInput,
};

#[tokio::test]
async fn test_analytics_tracking_and_pii_redaction() {
    let local_provider = Arc::new(LocalAnalyticsProvider::new());
    let posthog_provider = Arc::new(PostHogAnalyticsProvider::new(PostHogConfig {
        api_key: "ph_live_mock_123".to_string(),
        host: "https://app.posthog.com".to_string(),
    }));

    let service = AnalyticsService::new();
    service.register_provider(local_provider.clone());
    service.register_provider(posthog_provider.clone());

    // 1. Identify user
    service
        .identify_user(IdentifyUserInput {
            distinct_id: "cus_alice_99".to_string(),
            traits: serde_json::json!({
                "email": "alice@example.com",
                "plan": "pro"
            }),
        })
        .await
        .unwrap();

    // 2. Track order placed with sensitive properties
    let event = service
        .track_event(TrackEventInput {
            event_name: "order.completed".to_string(),
            distinct_id: "cus_alice_99".to_string(),
            properties: serde_json::json!({
                "order_id": "ord_9901",
                "revenue": 149.99,
                "email": "alice@example.com",
                "card_number": "4111222233334444",
                "secret_key": "top_secret_token"
            }),
        })
        .await
        .unwrap();

    assert_eq!(event.event_name, "order.completed");

    // 3. Verify Local provider received event
    let local_events = local_provider.get_events();
    assert_eq!(local_events.len(), 2);

    // 4. Verify PostHog provider REDACTED sensitive PII per SECURITY §11.1
    let posthog_events = posthog_provider.get_sent_events();
    assert_eq!(posthog_events.len(), 2);
    let tracked_order = &posthog_events[1];
    let props = &tracked_order.properties;

    assert_eq!(props["card_number"], "[REDACTED]");
    assert_eq!(props["secret_key"], "[REDACTED]");
    assert_eq!(props["email"], "a***@example.com"); // Pseudonymized
    assert_eq!(props["order_id"], "ord_9901");
    assert_eq!(props["revenue"], 149.99);
}
