use serde_json::json;
use stheno_events::{Event, EventBus, InMemoryEventBus};

#[tokio::test]
async fn test_in_memory_event_bus_emit_and_history() {
    let bus = InMemoryEventBus::new();

    let event1 = Event::new("product.created", json!({ "id": "prod_123" }));
    let event2 = Event::new(
        "product.updated",
        json!({ "id": "prod_123", "title": "New Title" }),
    );

    bus.emit(vec![event1.clone(), event2.clone()])
        .await
        .expect("emit must succeed");

    let history = bus.recorded_events();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].name, "product.created");
    assert_eq!(history[1].name, "product.updated");
}

#[tokio::test]
async fn test_in_memory_event_bus_subscription() {
    let bus = InMemoryEventBus::new();
    let mut rx = bus.subscribe("order.placed");

    let event = Event::new(
        "order.placed",
        json!({ "order_id": "ord_999", "total": "50.00" }),
    );
    bus.emit(vec![event.clone()])
        .await
        .expect("emit must succeed");

    let received = rx.recv().await.expect("must receive event");
    assert_eq!(received.name, "order.placed");
    assert_eq!(received.payload["order_id"], "ord_999");
}
