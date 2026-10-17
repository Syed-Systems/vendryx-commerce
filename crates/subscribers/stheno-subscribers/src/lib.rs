//! `stheno-subscribers`: Dedicated subscriber crate for cross-module reactive event handlers (AGENTS.md §2.1).

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use stheno_events::{Event, EventBus};
use stheno_types::Result;
pub mod identity_subscriber;
pub mod push_order_placed;

pub use identity_subscriber::{
    IdentityCreatedSubscriber, OrderConfirmationEmailSubscriber, PasswordResetSubscriber,
    UserInviteSubscriber,
};
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
                // Retry up to 3 times with a short delay, then log dead-letter 
                // rather than aborting all subscribers.
                let mut attempts = 0;
                loop {
                    attempts += 1;
                    match sub.handle(event).await {
                        Ok(()) => break,
                        Err(e) if attempts < 3 => {
                            tracing::warn!(
                                subscriber = %sub.event_name(),
                                event = %event.name,
                                attempt = attempts,
                                error = %e,
                                "subscriber failed; retrying"
                            );
                            tokio::time::sleep(Duration::from_millis(250)).await;
                        }
                        Err(e) => {
                            tracing::error!(
                                subscriber = %sub.event_name(),
                                event = %event.name,
                                error = %e,
                                "dead-letter: subscriber failed after 3 retries"
                            );
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Attaches `subscriber` to the bus as detached background work.
    ///
    /// The handler is re-entered into the emitting tenant's scope (ADR-012). A
    /// task-local does not survive `tokio::spawn`, so without this every subscriber that
    /// reaches the database would run unscoped — which under RLS means silently seeing
    /// nothing. An event carrying no tenant is skipped rather than handled ambiently:
    /// there is no correct tenant to guess.
    pub fn attach_subscriber(&self, bus: &dyn EventBus, subscriber: Arc<dyn EventSubscriber>) {
        let event_name = subscriber.event_name();
        let mut rx = bus.subscribe(event_name);
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if event.tenant_id.is_empty() {
                    tracing::warn!(
                        event = %event.name,
                        id = %event.id,
                        "skipping event with no tenant; it was emitted outside a tenant scope"
                    );
                    continue;
                }
                let tenant = event.tenant_id.clone();
                let sub = subscriber.clone();
                let _ =
                    stheno_db::tenant::scope(tenant, async move { sub.handle(&event).await }).await;
            }
        });
    }
}

impl Default for SubscriberRegistry {
    fn default() -> Self {
        Self::new()
    }
}
