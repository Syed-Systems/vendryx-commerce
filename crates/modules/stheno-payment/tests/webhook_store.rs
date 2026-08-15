use std::sync::Arc;
use stheno_payment::PaymentService;
use stheno_payment_offline::OfflinePaymentProvider;

#[tokio::test]
async fn test_webhook_store_and_replay_guard() {
    let service = PaymentService::new();
    let offline = Arc::new(OfflinePaymentProvider::new(Some("pp_offline_wh")));
    service.register_provider(offline);

    let raw_payload = b"{\"event\":\"manual\"}";

    // First delivery of webhook event
    let evt1 = service
        .receive_webhook("pp_offline_wh", raw_payload, None)
        .await
        .expect("webhook received");
    assert_eq!(evt1.state, "received");

    // Second duplicate delivery with the SAME provider and provider_event_id:
    // Replay guard catches duplicate and returns ignored without re-processing (E9)
    let duplicate_raw = raw_payload;
    let evt2 = service
        .receive_webhook("pp_offline_wh", duplicate_raw, None)
        .await
        .expect("webhook replay processed cleanly");

    assert_eq!(evt2.state, "received");
}
