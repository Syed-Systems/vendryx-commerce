use async_trait::async_trait;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use stheno_events::Event;
use stheno_subscribers::{EventSubscriber, SubscriberRegistry};
use stheno_types::Result;

struct OrderPlacedSubscriber {
    counter: Arc<AtomicUsize>,
}

#[async_trait]
impl EventSubscriber for OrderPlacedSubscriber {
    fn event_name(&self) -> &'static str {
        "order.placed"
    }

    async fn handle(&self, _event: &Event) -> Result<()> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_subscriber_registration_and_event_handling() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut registry = SubscriberRegistry::new();
    registry.register(Arc::new(OrderPlacedSubscriber {
        counter: counter.clone(),
    }));

    let event = Event::new("order.placed", json!({ "total": 5000 }));
    let _ = registry.dispatch(&event).await;

    assert_eq!(counter.load(Ordering::SeqCst), 1);
}
