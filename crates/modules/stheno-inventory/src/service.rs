//! InventoryService with atomic reservations and oversell prevention, backed by Postgres (P12-07, P12-08).

use chrono::Utc;
use stheno_db::diesel::{self, sql_query};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{
    with_transaction, CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection,
    PgCrudRepository,
};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{InventoryItem, InventoryLevel, ReservationItem};

#[derive(Clone)]
pub struct InventoryService {
    pool: DbPool,
    items: PgCrudRepository<InventoryItem>,
    levels: PgCrudRepository<InventoryLevel>,
    reservations: PgCrudRepository<ReservationItem>,
}

impl InventoryService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            items: PgCrudRepository::new(pool.clone()),
            levels: PgCrudRepository::new(pool.clone()),
            reservations: PgCrudRepository::new(pool.clone()),
            pool,
        }
    }

    pub async fn create_item(&self, input: CreateInventoryItemInput) -> Result<InventoryItemDTO> {
        let item_id = Id::new();
        let now = Utc::now();
        let item = InventoryItem {
            id: item_id,
            sku: input.sku,
            hs_code: None,
            origin_country: None,
            mid_code: None,
            material: None,
            requires_shipping: input.requires_shipping.unwrap_or(true),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.items.create(None, item).await?;

        Ok(InventoryItemDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            sku: created.sku,
            requires_shipping: created.requires_shipping,
            created_at: created.created_at,
        })
    }

    pub async fn create_level(
        &self,
        input: CreateInventoryLevelInput,
    ) -> Result<InventoryLevelDTO> {
        let level_id = Id::new();
        let now = Utc::now();

        let level = InventoryLevel {
            id: level_id,
            inventory_item_id: input.inventory_item_id.clone(),
            location_id: input.location_id.clone(),
            stocked_quantity: input.stocked_quantity,
            reserved_quantity: 0,
            incoming_quantity: 0,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.levels.create(None, level).await?;

        Ok(InventoryLevelDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            inventory_item_id: created.inventory_item_id,
            location_id: created.location_id,
            stocked_quantity: created.stocked_quantity,
            reserved_quantity: created.reserved_quantity,
            available_quantity: created.stocked_quantity - created.reserved_quantity,
        })
    }

    pub async fn get_level(
        &self,
        inventory_item_id: &str,
        location_id: &str,
    ) -> Result<InventoryLevelDTO> {
        let filter = FilterSpec::new()
            .with_predicate(
                "inventory_item_id",
                FilterOp::Eq(inventory_item_id.to_string()),
            )
            .with_predicate("location_id", FilterOp::Eq(location_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let levels = self.levels.list(None, &filter).await?;
        let level = levels.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "inventory_level",
            id: format!("{}::{}", inventory_item_id, location_id),
        })?;

        Ok(InventoryLevelDTO {
            id: Id::from_string(level.id.to_string()).unwrap_or_default(),
            inventory_item_id: level.inventory_item_id,
            location_id: level.location_id,
            stocked_quantity: level.stocked_quantity,
            reserved_quantity: level.reserved_quantity,
            available_quantity: level.stocked_quantity - level.reserved_quantity,
        })
    }

    /// Atomically reserves quantity against an inventory level (P12-07, P12-08).
    /// Uses Postgres atomic conditional update to guarantee no overselling under concurrency.
    pub async fn reserve(&self, input: CreateReservationInput) -> Result<ReservationItemDTO> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let res_id = Id::new();
        let now = Utc::now();
        let reservation = ReservationItem {
            id: res_id,
            line_item_id: input.line_item_id.clone(),
            inventory_item_id: input.inventory_item_id.clone(),
            location_id: input.location_id.clone(),
            quantity: input.quantity,
            description: input.description.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let reservations = self.reservations.clone();
        let item_id = input.inventory_item_id.clone();
        let loc_id = input.location_id.clone();
        let qty = input.quantity;

        with_transaction(&mut conn, move |mut c| {
            Box::pin(async move {
                // Atomic conditional update on inventory_level
                let update_query = sql_query(
                    "UPDATE inventory.inventory_level \
                     SET reserved_quantity = reserved_quantity + $1, updated_at = now() \
                     WHERE inventory_item_id = $2 AND location_id = $3 AND deleted_at IS NULL \
                     AND (stocked_quantity - reserved_quantity) >= $1",
                )
                .bind::<diesel::sql_types::Integer, _>(qty)
                .bind::<diesel::sql_types::Text, _>(&item_id)
                .bind::<diesel::sql_types::Text, _>(&loc_id);

                let updated_rows = update_query
                    .execute(&mut c)
                    .await
                    .map_err(|e| Error::Database(e.to_string()))?;

                if updated_rows == 0 {
                    return Err(Error::Conflict {
                        code: "insufficient_inventory",
                        message: format!("Insufficient inventory: requested {}", qty),
                    });
                }

                let created_res = reservations.create(Some(&mut c), reservation).await?;
                Ok(ReservationItemDTO {
                    id: Id::from_string(created_res.id.to_string()).unwrap_or_default(),
                    line_item_id: created_res.line_item_id,
                    inventory_item_id: created_res.inventory_item_id,
                    location_id: created_res.location_id,
                    quantity: created_res.quantity,
                })
            })
        })
        .await
    }

    /// Releases a reservation upon cancellation
    pub async fn release_reservation(&self, reservation_id: &str) -> Result<()> {
        let reservation = self.reservations.retrieve(None, reservation_id).await?;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let reservations = self.reservations.clone();
        let resid = reservation.id.to_string();
        let item_id = reservation.inventory_item_id.clone();
        let loc_id = reservation.location_id.clone();
        let qty = reservation.quantity;

        with_transaction(&mut conn, move |mut c| {
            Box::pin(async move {
                sql_query(
                    "UPDATE inventory.inventory_level \
                     SET reserved_quantity = GREATEST(0, reserved_quantity - $1), updated_at = now() \
                     WHERE inventory_item_id = $2 AND location_id = $3",
                )
                .bind::<diesel::sql_types::Integer, _>(qty)
                .bind::<diesel::sql_types::Text, _>(&item_id)
                .bind::<diesel::sql_types::Text, _>(&loc_id)
                .execute(&mut c)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                reservations.delete(Some(&mut c), &resid).await?;
                Ok(())
            })
        })
        .await
    }

    /// Fulfills a reservation by decrementing both stocked and reserved quantities
    pub async fn fulfill_reservation(&self, reservation_id: &str) -> Result<()> {
        let reservation = self.reservations.retrieve(None, reservation_id).await?;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        let reservations = self.reservations.clone();
        let resid = reservation.id.to_string();
        let item_id = reservation.inventory_item_id.clone();
        let loc_id = reservation.location_id.clone();
        let qty = reservation.quantity;

        with_transaction(&mut conn, move |mut c| {
            Box::pin(async move {
                sql_query(
                    "UPDATE inventory.inventory_level \
                     SET stocked_quantity = GREATEST(0, stocked_quantity - $1), \
                         reserved_quantity = GREATEST(0, reserved_quantity - $1), \
                         updated_at = now() \
                     WHERE inventory_item_id = $2 AND location_id = $3",
                )
                .bind::<diesel::sql_types::Integer, _>(qty)
                .bind::<diesel::sql_types::Text, _>(&item_id)
                .bind::<diesel::sql_types::Text, _>(&loc_id)
                .execute(&mut c)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                reservations.delete(Some(&mut c), &resid).await?;
                Ok(())
            })
        })
        .await
    }

    pub async fn get_available_quantity(&self, item_id: &str, location_id: &str) -> Result<i32> {
        let level = self.get_level(item_id, location_id).await?;
        Ok(level.available_quantity)
    }

    pub async fn list_items(&self) -> Result<Vec<InventoryItemDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_order("created_at", OrderDirection::Asc)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let items = self.items.list(None, &filter).await?;
        Ok(items
            .into_iter()
            .map(|i| InventoryItemDTO {
                id: Id::from_string(i.id.to_string()).unwrap_or_default(),
                sku: i.sku,
                requires_shipping: i.requires_shipping,
                created_at: i.created_at,
            })
            .collect())
    }
}
