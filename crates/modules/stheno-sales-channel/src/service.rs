//! SalesChannelService domain service, backed by Postgres (P12-10).

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Id, Result};

use crate::dto::*;
use crate::models::SalesChannel;

#[derive(Clone)]
pub struct SalesChannelService {
    channels: PgCrudRepository<SalesChannel>,
}

impl SalesChannelService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            channels: PgCrudRepository::new(pool),
        }
    }

    pub async fn create_channel(&self, input: CreateSalesChannelInput) -> Result<SalesChannelDTO> {
        let sc_id = Id::new();
        let now = Utc::now();
        let sc = SalesChannel {
            id: sc_id,
            name: input.name,
            description: input.description,
            is_disabled: input.is_disabled.unwrap_or(false),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.channels.create(None, sc).await?;

        Ok(SalesChannelDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            name: created.name,
            description: created.description,
            is_disabled: created.is_disabled,
            created_at: created.created_at,
        })
    }

    pub async fn get_channel(&self, id: &str) -> Result<SalesChannelDTO> {
        let sc = self.channels.retrieve(None, id).await?;

        Ok(SalesChannelDTO {
            id: Id::from_string(sc.id.to_string()).unwrap_or_default(),
            name: sc.name,
            description: sc.description,
            is_disabled: sc.is_disabled,
            created_at: sc.created_at,
        })
    }

    pub async fn list_channels(&self) -> Result<Vec<SalesChannelDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.channels.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|sc| SalesChannelDTO {
                id: Id::from_string(sc.id.to_string()).unwrap_or_default(),
                name: sc.name,
                description: sc.description,
                is_disabled: sc.is_disabled,
                created_at: sc.created_at,
            })
            .collect())
    }

    /// Partial update: an absent field is "unchanged", not "set to null".
    ///
    /// `description` is nullable in the model, so the dashboard cannot clear it through
    /// this shape. That is the correct trade for a partial update — a clear needs its own
    /// signal, and nothing asks for one yet.
    pub async fn update_channel(
        &self,
        id: &str,
        input: UpdateSalesChannelInput,
    ) -> Result<SalesChannelDTO> {
        let mut sc = self.channels.retrieve(None, id).await?;

        if let Some(name) = input.name {
            sc.name = name;
        }
        if let Some(description) = input.description {
            sc.description = Some(description);
        }
        if let Some(is_disabled) = input.is_disabled {
            sc.is_disabled = is_disabled;
        }
        sc.updated_at = Utc::now();

        let updated = self.channels.update(None, id, sc).await?;

        Ok(SalesChannelDTO {
            id: Id::from_string(updated.id.to_string()).unwrap_or_default(),
            name: updated.name,
            description: updated.description,
            is_disabled: updated.is_disabled,
            created_at: updated.created_at,
        })
    }

    pub async fn delete_channel(&self, id: &str) -> Result<()> {
        let sc = self.channels.retrieve(None, id).await?;
        self.channels.soft_delete(None, &sc.id.to_string()).await
    }
}
