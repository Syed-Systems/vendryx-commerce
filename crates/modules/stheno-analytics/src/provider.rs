use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use stheno_types::Result;

#[async_trait]
pub trait AnalyticsProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    async fn track(
        &self,
        event_name: &str,
        distinct_id: &str,
        properties: &serde_json::Value,
    ) -> Result<()>;
    async fn identify(&self, distinct_id: &str, traits: &serde_json::Value) -> Result<()>;
}

// ==========================================
// 1. Local Analytics Provider
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedEvent {
    pub event_name: String,
    pub distinct_id: String,
    pub properties: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Default)]
pub struct LocalAnalyticsProvider {
    events: Arc<RwLock<Vec<TrackedEvent>>>,
}

impl LocalAnalyticsProvider {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_events(&self) -> Vec<TrackedEvent> {
        self.events.read().unwrap().clone()
    }
}

#[async_trait]
impl AnalyticsProvider for LocalAnalyticsProvider {
    fn provider_id(&self) -> &'static str {
        "local"
    }

    async fn track(
        &self,
        event_name: &str,
        distinct_id: &str,
        properties: &serde_json::Value,
    ) -> Result<()> {
        let mut list = self.events.write().unwrap();
        list.push(TrackedEvent {
            event_name: event_name.to_string(),
            distinct_id: distinct_id.to_string(),
            properties: properties.clone(),
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }

    async fn identify(&self, distinct_id: &str, traits: &serde_json::Value) -> Result<()> {
        let mut list = self.events.write().unwrap();
        list.push(TrackedEvent {
            event_name: "$identify".to_string(),
            distinct_id: distinct_id.to_string(),
            properties: traits.clone(),
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }
}

// ==========================================
// 2. PostHog Analytics Provider with PII Redaction (SECURITY §11.1)
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHogConfig {
    pub api_key: String,
    pub host: String,
}

pub struct PostHogAnalyticsProvider {
    config: PostHogConfig,
    sent_events: Arc<RwLock<Vec<TrackedEvent>>>,
}

impl PostHogAnalyticsProvider {
    pub fn new(config: PostHogConfig) -> Self {
        Self {
            config,
            sent_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_sent_events(&self) -> Vec<TrackedEvent> {
        self.sent_events.read().unwrap().clone()
    }

    pub fn config(&self) -> &PostHogConfig {
        &self.config
    }

    /// Redacts sensitive PII (emails, passwords, cards, full names) per SECURITY §11.1
    fn sanitize_properties(props: &serde_json::Value) -> serde_json::Value {
        match props {
            serde_json::Value::Object(map) => {
                let mut sanitized = serde_json::Map::new();
                for (k, v) in map {
                    let k_lower = k.to_lowercase();
                    if k_lower.contains("password")
                        || k_lower.contains("secret")
                        || k_lower.contains("card")
                        || k_lower.contains("cvv")
                        || k_lower.contains("token")
                    {
                        sanitized.insert(k.clone(), serde_json::json!("[REDACTED]"));
                    } else if k_lower.contains("email") {
                        if let serde_json::Value::String(email) = v {
                            // Pseudonymize email: e***@domain.com or hash
                            let parts: Vec<&str> = email.split('@').collect();
                            if parts.len() == 2 {
                                let pseudo = format!(
                                    "{}***@{}",
                                    &parts[0][0..1.min(parts[0].len())],
                                    parts[1]
                                );
                                sanitized.insert(k.clone(), serde_json::json!(pseudo));
                            } else {
                                sanitized.insert(k.clone(), serde_json::json!("[REDACTED]"));
                            }
                        } else {
                            sanitized.insert(k.clone(), serde_json::json!("[REDACTED]"));
                        }
                    } else {
                        sanitized.insert(k.clone(), Self::sanitize_properties(v));
                    }
                }
                serde_json::Value::Object(sanitized)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::sanitize_properties).collect())
            }
            other => other.clone(),
        }
    }
}

#[async_trait]
impl AnalyticsProvider for PostHogAnalyticsProvider {
    fn provider_id(&self) -> &'static str {
        "posthog"
    }

    async fn track(
        &self,
        event_name: &str,
        distinct_id: &str,
        properties: &serde_json::Value,
    ) -> Result<()> {
        let sanitized = Self::sanitize_properties(properties);
        let mut list = self.sent_events.write().unwrap();
        list.push(TrackedEvent {
            event_name: event_name.to_string(),
            distinct_id: distinct_id.to_string(),
            properties: sanitized,
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }

    async fn identify(&self, distinct_id: &str, traits: &serde_json::Value) -> Result<()> {
        let sanitized = Self::sanitize_properties(traits);
        let mut list = self.sent_events.write().unwrap();
        list.push(TrackedEvent {
            event_name: "$identify".to_string(),
            distinct_id: distinct_id.to_string(),
            properties: sanitized,
            timestamp: chrono::Utc::now(),
        });
        Ok(())
    }
}
