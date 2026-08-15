use serde_json::json;
use stheno_events::{Event, EventBus, InMemoryEventBus};

#[tokio::test]
async fn test_product_domain_events_emission() {
    let bus = InMemoryEventBus::new();
    let mut rx = bus.subscribe("product.created");

    let event = Event::new(
        "product.created",
        json!({
            "id": "prod_01JTEST",
            "title": "Classic Tee"
        }),
    );

    bus.emit(vec![event.clone()]).await.expect("emit event");
    let received = rx.recv().await.expect("receive event");

    assert_eq!(received.name, "product.created");
    assert_eq!(received.payload["id"], "prod_01JTEST");
}
