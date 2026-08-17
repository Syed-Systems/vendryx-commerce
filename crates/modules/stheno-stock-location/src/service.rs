//! StockLocationService domain service, backed by Postgres (P12-09).

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Id, Result};

use crate::dto::*;
use crate::models::StockLocation;

#[derive(Clone)]
pub struct StockLocationService {
    locations: PgCrudRepository<StockLocation>,
}

impl StockLocationService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            locations: PgCrudRepository::new(pool),
        }
    }

    pub async fn create_location(
        &self,
        input: CreateStockLocationInput,
    ) -> Result<StockLocationDTO> {
        let loc_id = Id::new();
        let now = Utc::now();
        let loc = StockLocation {
            id: loc_id,
            name: input.name,
            address_id: input.address_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.locations.create(None, loc).await?;

        Ok(StockLocationDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            name: created.name,
            address_id: created.address_id,
            created_at: created.created_at,
        })
    }

    pub async fn get_location(&self, id: &str) -> Result<StockLocationDTO> {
        let loc = self.locations.retrieve(None, id).await?;

        Ok(StockLocationDTO {
            id: Id::from_string(loc.id.to_string()).unwrap_or_default(),
            name: loc.name,
            address_id: loc.address_id,
            created_at: loc.created_at,
        })
    }

    pub async fn list_locations(&self) -> Result<Vec<StockLocationDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let list = self.locations.list(None, &filter).await?;
        Ok(list
            .into_iter()
            .map(|l| StockLocationDTO {
                id: Id::from_string(l.id.to_string()).unwrap_or_default(),
                name: l.name,
                address_id: l.address_id,
                created_at: l.created_at,
            })
            .collect())
    }

    pub async fn delete_location(&self, id: &str) -> Result<()> {
        let loc = self.locations.retrieve(None, id).await?;
        self.locations.soft_delete(None, &loc.id.to_string()).await
    }
}
