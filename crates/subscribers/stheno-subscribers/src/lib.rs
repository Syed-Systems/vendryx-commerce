//! `stheno-subscribers`: Dedicated subscriber crate for cross-module reactive event handlers (AGENTS.md §2.1).

use async_trait::async_trait;
use std::sync::Arc;
use stheno_events::{Event, EventBus};
use stheno_types::Result;
pub mod push_order_placed;

pub use push_order_placed::PushOrderPlacedSubscriber;

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    fn event_name(&self) -> &'static str;
    async fn handle(&self, event: &Event) -> Result<()>;
}

pub struct SubscriberRegistry {
    subscribers: Vec<Arc<dyn EventSubscriber>>,
}

impl SubscriberRegistry {
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    pub fn register(&mut self, subscriber: Arc<dyn EventSubscriber>) {
        self.subscribers.push(subscriber);
    }

    pub async fn dispatch(&self, event: &Event) -> Result<()> {
        for sub in &self.subscribers {
            if sub.event_name() == event.name || sub.event_name() == "*" {
                sub.handle(event).await?;
            }
        }
        Ok(())
    }

    pub fn attach_subscriber(&self, bus: &dyn EventBus, subscriber: Arc<dyn EventSubscriber>) {
        let event_name = subscriber.event_name();
        let mut rx = bus.subscribe(event_name);
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let _ = subscriber.handle(&event).await;
            }
        });
    }
}

impl Default for SubscriberRegistry {
    fn default() -> Self {
        Self::new()
    }
}
