use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Id, Result};

use crate::dto::{AnalyticsEventDTO, IdentifyUserInput, TrackEventInput};
use crate::models::AnalyticsEvent;
use crate::provider::AnalyticsProvider;

#[derive(Default)]
struct ProviderRegistry(HashMap<String, Arc<dyn AnalyticsProvider>>);

#[derive(Clone)]
pub struct AnalyticsService {
    providers: Arc<RwLock<ProviderRegistry>>,
    events: PgCrudRepository<AnalyticsEvent>,
}

impl AnalyticsService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            providers: Arc::new(RwLock::new(ProviderRegistry(HashMap::new()))),
            events: PgCrudRepository::new(pool),
        }
    }

    pub fn register_provider(&self, provider: Arc<dyn AnalyticsProvider>) {
        let mut reg = self.providers.write().unwrap();
        reg.0.insert(provider.provider_id().to_string(), provider);
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
            deleted_at: None,
        };

        let dto = AnalyticsEventDTO {
            id: id.to_string(),
            event_name: event.event_name.clone(),
            distinct_id: event.distinct_id.clone(),
            properties: event.properties.clone(),
            timestamp: event.timestamp,
        };

        // Broadcast to all registered analytics providers
        let providers: Vec<Arc<dyn AnalyticsProvider>> = {
            let reg = self.providers.read().unwrap();
            reg.0.values().cloned().collect()
        };
        for provider in &providers {
            let _ = provider
                .track(&input.event_name, &input.distinct_id, &input.properties)
                .await;
        }

        self.events.create(None, event).await?;

        Ok(dto)
    }

    pub async fn identify_user(&self, input: IdentifyUserInput) -> Result<()> {
        let providers: Vec<Arc<dyn AnalyticsProvider>> = {
            let reg = self.providers.read().unwrap();
            reg.0.values().cloned().collect()
        };
        for provider in &providers {
            let _ = provider.identify(&input.distinct_id, &input.traits).await;
        }
        Ok(())
    }

    pub async fn list_events_for_distinct_id(
        &self,
        distinct_id: &str,
    ) -> Result<Vec<AnalyticsEventDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("distinct_id", FilterOp::Eq(distinct_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("timestamp", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.events.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|e| AnalyticsEventDTO {
                id: e.id.to_string(),
                event_name: e.event_name,
                distinct_id: e.distinct_id,
                properties: e.properties,
                timestamp: e.timestamp,
            })
            .collect())
    }
}
