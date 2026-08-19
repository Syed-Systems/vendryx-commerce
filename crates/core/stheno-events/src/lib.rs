//! `stheno-events`: EventBus trait, in-memory event bus, and transactional outbox infrastructure.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stheno_types::Result;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    /// The tenant this event belongs to (ADR-012).
    ///
    /// Carried on the event because subscribers run in a detached `tokio::spawn`, and a
    /// task-local does not cross that boundary. Without it every subscriber that touches
    /// the database would fail closed.
    ///
    /// `serde(default)` so an outbox row written before this field existed still
    /// deserializes — it yields an empty tenant, which fails closed rather than being
    /// silently attributed to whoever happens to be in scope.
    #[serde(default)]
    pub tenant_id: String,
}

impl Event {
    /// Captures the emitting tenant from the ambient scope, so publishers need no change.
    ///
    /// An event emitted with no tenant in scope gets an empty `tenant_id`; the subscriber
    /// side refuses to dispatch it rather than guessing.
    pub fn new(name: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: ulid::Ulid::new().to_string(),
            name: name.into(),
            payload,
            timestamp: Utc::now(),
            tenant_id: stheno_db::tenant::current().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub id: String,
    pub event_name: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait EventBus: Send + Sync + 'static {
    async fn emit(&self, events: Vec<Event>) -> Result<()>;
    fn subscribe(&self, event_name: &str) -> broadcast::Receiver<Event>;
}

/// Thread-safe in-memory event bus with broadcast and history recording for testing and local development.
#[derive(Clone, Default)]
pub struct InMemoryEventBus {
    history: Arc<Mutex<Vec<Event>>>,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<Event>>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(Vec::new())),
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn recorded_events(&self) -> Vec<Event> {
        self.history.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.history.lock().unwrap().clear();
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn emit(&self, events: Vec<Event>) -> Result<()> {
        let mut hist = self.history.lock().unwrap();
        let channels = self.channels.lock().unwrap();

        for event in events {
            if let Some(sender) = channels.get(&event.name) {
                let _ = sender.send(event.clone());
            }
            hist.push(event);
        }

        Ok(())
    }

    fn subscribe(&self, event_name: &str) -> broadcast::Receiver<Event> {
        let mut channels = self.channels.lock().unwrap();
        let sender = channels
            .entry(event_name.to_string())
            .or_insert_with(|| broadcast::channel(128).0);
        sender.subscribe()
    }
}
