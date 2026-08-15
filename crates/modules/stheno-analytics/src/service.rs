use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use stheno_types::{Id, Result};

use crate::dto::{AnalyticsEventDTO, IdentifyUserInput, TrackEventInput};
use crate::models::AnalyticsEvent;
use crate::provider::AnalyticsProvider;

pub struct AnalyticsService {
    providers: RwLock<HashMap<String, Arc<dyn AnalyticsProvider>>>,
    events: RwLock<HashMap<String, AnalyticsEvent>>,
}

impl Default for AnalyticsService {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyticsService {
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
            events: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn AnalyticsProvider>) {
        let mut map = self.providers.write().unwrap();
        map.insert(provider.provider_id().to_string(), provider);
    }

    pub async fn track_event(&self, input: TrackEventInput) -> Result<AnalyticsEventDTO> {
        let id = Id::new();
        let now = Utc::now();

        let event = AnalyticsEvent {
            id,
            event_name: input.event_name.clone(),
            distinct_id: input.distinct_id.clone(),
            properties: input.properties.clone(),
            timestamp: now,
        };

        let dto = AnalyticsEventDTO {
            id: id.to_string(),
            event_name: event.event_name.clone(),
            distinct_id: event.distinct_id.clone(),
            properties: event.properties.clone(),
            timestamp: event.timestamp,
        };

        // Broadcast to all registered analytics providers
        let map = self.providers.read().unwrap();
        for provider in map.values() {
            let _ = provider
                .track(&input.event_name, &input.distinct_id, &input.properties)
                .await;
        }

        {
            let mut ev_map = self.events.write().unwrap();
            ev_map.insert(id.to_string(), event);
        }

        Ok(dto)
    }

    pub async fn identify_user(&self, input: IdentifyUserInput) -> Result<()> {
        let map = self.providers.read().unwrap();
        for provider in map.values() {
            let _ = provider.identify(&input.distinct_id, &input.traits).await;
        }
        Ok(())
    }

    pub async fn list_events_for_distinct_id(
        &self,
        distinct_id: &str,
    ) -> Result<Vec<AnalyticsEventDTO>> {
        let map = self.events.read().unwrap();
        let list: Vec<AnalyticsEventDTO> = map
            .values()
            .filter(|e| e.distinct_id == distinct_id)
            .map(|e| AnalyticsEventDTO {
                id: e.id.to_string(),
                event_name: e.event_name.clone(),
                distinct_id: e.distinct_id.clone(),
                properties: e.properties.clone(),
                timestamp: e.timestamp,
            })
            .collect();
        Ok(list)
    }
}
