//! InventoryService with atomic reservations and oversell prevention, backed by Postgres (P12-07, P12-08).

use chrono::Utc;
use stheno_db::diesel::QueryableByName;
use stheno_db::diesel::result::OptionalExtension;
use stheno_db::diesel::sql_types::{Integer, Nullable, Text};
use stheno_db::diesel::{self, sql_query};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::filter::MAX_PAGE_LIMIT;
use stheno_db::tenant;
use stheno_db::{
    CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository,
    with_transaction,
};

use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{InventoryItem, InventoryLevel, ReservationItem};

#[derive(QueryableByName)]
struct ExistingReservationRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Nullable<Text>)]
    line_item_id: Option<String>,
    #[diesel(sql_type = Text)]
    inventory_item_id: String,
    #[diesel(sql_type = Text)]
    location_id: String,
    #[diesel(sql_type = Integer)]
    quantity: i32,
}

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
            id: format!("{inventory_item_id}::{location_id}"),
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
        if input.quantity <= 0 {
            return Err(Error::Invalid {
                code: "reservation_quantity_invalid",
                message: "Reservation quantity must be positive.".to_string(),
                field: Some("quantity".to_string()),
            });
        }
        // Cloned out of `self` before the closure: `with_tenant_conn` moves it into a
        // 'static future, which cannot borrow `&self`.
        let reservations = self.reservations.clone();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {


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

                let item_id = input.inventory_item_id.clone();
                let loc_id = input.location_id.clone();
                let qty = input.quantity;

                with_transaction(conn, move |mut c| {
                    Box::pin(async move {
                        if let Some(line_item_id) = input.line_item_id.as_deref() {
                            sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                                .bind::<Text, _>(format!("reservation:{line_item_id}"))
                                .execute(&mut c)
                                .await
                                .map_err(|error| Error::Database(error.to_string()))?;
                            let existing = sql_query(
                                "SELECT id, line_item_id, inventory_item_id, location_id, quantity \
                                 FROM inventory.reservation_item \
                                 WHERE line_item_id = $1 AND deleted_at IS NULL LIMIT 1",
                            )
                            .bind::<Text, _>(line_item_id)
                            .get_result::<ExistingReservationRow>(&mut c)
                            .await
                            .optional()
                            .map_err(|error| Error::Database(error.to_string()))?;
                            if let Some(existing) = existing {
                                if existing.inventory_item_id != input.inventory_item_id
                                    || existing.location_id != input.location_id
                                    || existing.quantity != input.quantity
                                {
                                    return Err(Error::Conflict {
                                        code: "reservation_idempotency_conflict",
                                        message: "This order line already has a different active inventory reservation.".to_string(),
                                    });
                                }
                                return Ok(ReservationItemDTO {
                                    id: Id::from_string(existing.id).unwrap_or_default(),
                                    line_item_id: existing.line_item_id,
                                    inventory_item_id: existing.inventory_item_id,
                                    location_id: existing.location_id,
                                    quantity: existing.quantity,
                                });
                            }
                        }

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
                                message: format!("Insufficient inventory: requested {qty}"),
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

            })
        })
        .await
    }

    /// Releases a reservation upon cancellation
    pub async fn release_reservation(&self, reservation_id: &str) -> Result<()> {
        let reservation_id = reservation_id.to_string();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                with_transaction(conn, move |mut c| {
                    Box::pin(async move {
                        let Some(reservation) = sql_query(
                            "SELECT id, line_item_id, inventory_item_id, location_id, quantity \
                             FROM inventory.reservation_item \
                             WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
                        )
                        .bind::<Text, _>(&reservation_id)
                        .get_result::<ExistingReservationRow>(&mut c)
                        .await
                        .optional()
                        .map_err(|error| Error::Database(error.to_string()))?
                        else {
                            return Ok(());
                        };
                        sql_query(
                            "UPDATE inventory.inventory_level \
                             SET reserved_quantity = GREATEST(0, reserved_quantity - $1), updated_at = now() \
                             WHERE inventory_item_id = $2 AND location_id = $3",
                        )
                        .bind::<Integer, _>(reservation.quantity)
                        .bind::<Text, _>(&reservation.inventory_item_id)
                        .bind::<Text, _>(&reservation.location_id)
                        .execute(&mut c)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?;
                        sql_query(
                            "UPDATE inventory.reservation_item SET deleted_at = now(), updated_at = now() \
                             WHERE id = $1 AND deleted_at IS NULL",
                        )
                        .bind::<Text, _>(&reservation.id)
                        .execute(&mut c)
                        .await
                        .map_err(|error| Error::Database(error.to_string()))?;
                        Ok(())
                    })
                })
                .await

            })
        })
        .await
    }

    /// Fulfills a reservation by decrementing both stocked and reserved quantities
    pub async fn fulfill_reservation(&self, reservation_id: &str) -> Result<()> {
        let reservation_id = reservation_id.to_string();
        let reservations = self.reservations.clone();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let reservation = reservations.retrieve(None, &reservation_id).await?;

                let resid = reservation.id.to_string();
                let item_id = reservation.inventory_item_id.clone();
                let loc_id = reservation.location_id.clone();
                let qty = reservation.quantity;

                with_transaction(conn, move |mut c| {
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
            })
        })
        .await
    }

    pub async fn get_available_quantity(&self, item_id: &str, location_id: &str) -> Result<i32> {
        let level = self.get_level(item_id, location_id).await?;
        Ok(level.available_quantity)
    }

    /// Total availability per variant, summed across locations.
    ///
    /// The storefront disables add-to-cart for any variant that manages inventory and has
    /// no `inventory_quantity`, so without this every seeded product read "Out of stock"
    /// however much stock it actually had.
    ///
    /// Variants with no inventory item are absent rather than zero: "not stock-tracked"
    /// and "none left" are different answers, and conflating them hides one behind the
    /// other.
    pub async fn availability_for_variants(
        &self,
        variant_ids: &[String],
    ) -> Result<std::collections::HashMap<String, i32>> {
        if variant_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let links = stheno_links::PgLinkStore::new(self.pool.clone());
        let items_by_variant = links
            .get_by_left_many("product_variant_inventory_item", variant_ids)
            .await?;

        let item_ids: Vec<String> = items_by_variant.values().flatten().cloned().collect();
        if item_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let filter = FilterSpec::new()
            .with_predicate("inventory_item_id", FilterOp::In(item_ids))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        let mut available_by_item: std::collections::HashMap<String, i32> =
            std::collections::HashMap::new();
        for level in self.levels.list(None, &filter).await? {
            *available_by_item
                .entry(level.inventory_item_id.clone())
                .or_insert(0) += level.stocked_quantity - level.reserved_quantity;
        }

        let mut out = std::collections::HashMap::new();
        for (variant_id, item_ids) in items_by_variant {
            let total: i32 = item_ids
                .iter()
                .filter_map(|id| available_by_item.get(id))
                .sum();
            out.insert(variant_id, total);
        }
        Ok(out)
    }

    /// The inventory items behind each variant, with their location levels.
    ///
    /// The admin's product detail page reads
    /// `variant.inventory_items[].inventory.location_levels`. The field was never sent, so
    /// the page threw `Cannot read properties of undefined (reading 'length')` and no
    /// product with `manage_inventory` could be opened at all.
    pub async fn items_for_variants(
        &self,
        variant_ids: &[String],
    ) -> Result<std::collections::HashMap<String, Vec<(InventoryItemDTO, Vec<InventoryLevelDTO>)>>>
    {
        if variant_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let links = stheno_links::PgLinkStore::new(self.pool.clone());
        let items_by_variant = links
            .get_by_left_many("product_variant_inventory_item", variant_ids)
            .await?;

        let mut out = std::collections::HashMap::new();
        for (variant_id, item_ids) in items_by_variant {
            let mut items = Vec::with_capacity(item_ids.len());
            for item_id in item_ids {
                // ponytail: two round trips per inventory item. A product detail page holds
                // a handful of variants, so batch these only if kit-heavy catalogs make it
                // measurable.
                let Ok(item) = self.get_item(&item_id).await else {
                    continue;
                };
                let levels = self.list_levels(&item_id).await.unwrap_or_default();
                items.push((item, levels));
            }
            out.insert(variant_id, items);
        }
        Ok(out)
    }

    /// One inventory item.
    ///
    /// The API answered this with a mock JSON object carrying a fabricated SKU, so the
    /// admin's inventory detail screen showed the same invented item for every id.
    pub async fn get_item(&self, id: &str) -> Result<InventoryItemDTO> {
        let item = self.items.retrieve(None, id).await?;
        Ok(InventoryItemDTO {
            id: Id::from_string(item.id.to_string()).unwrap_or_default(),
            sku: item.sku,
            requires_shipping: item.requires_shipping,
            created_at: item.created_at,
        })
    }

    /// Every stock level for an item, across locations.
    pub async fn list_levels(&self, item_id: &str) -> Result<Vec<InventoryLevelDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("inventory_item_id", FilterOp::Eq(item_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);

        Ok(self
            .levels
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|l| InventoryLevelDTO {
                available_quantity: l.stocked_quantity - l.reserved_quantity,
                id: Id::from_string(l.id.to_string()).unwrap_or_default(),
                inventory_item_id: l.inventory_item_id,
                location_id: l.location_id,
                stocked_quantity: l.stocked_quantity,
                reserved_quantity: l.reserved_quantity,
            })
            .collect())
    }

    /// Adjusts stocked quantity at a location — a receipt, a stock count, or shrinkage.
    ///
    /// This is the single most important operation in inventory management and there was
    /// no way to perform it: a level could be created once and never changed again.
    ///
    /// The new quantity may not drop below what is already reserved. Allowing it would
    /// make `available_quantity` negative and let the next reservation oversell against
    /// stock that is already promised to an order.
    pub async fn set_stocked_quantity(
        &self,
        item_id: &str,
        location_id: &str,
        stocked_quantity: i32,
    ) -> Result<InventoryLevelDTO> {
        if stocked_quantity < 0 {
            return Err(Error::Invalid {
                code: "negative_stock",
                message: "stocked quantity cannot be negative".to_string(),
                field: Some("stocked_quantity".to_string()),
            });
        }

        let filter = FilterSpec::new()
            .with_predicate("inventory_item_id", FilterOp::Eq(item_id.to_string()))
            .with_predicate("location_id", FilterOp::Eq(location_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let mut level = self
            .levels
            .list(None, &filter)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound {
                entity: "inventory_level",
                id: format!("{item_id}@{location_id}"),
            })?;

        if stocked_quantity < level.reserved_quantity {
            return Err(Error::Invalid {
                code: "stock_below_reserved",
                message: format!(
                    "cannot set stock to {stocked_quantity}; {} is already reserved",
                    level.reserved_quantity
                ),
                field: Some("stocked_quantity".to_string()),
            });
        }

        let id = level.id.to_string();
        level.stocked_quantity = stocked_quantity;
        level.updated_at = Utc::now();
        let saved = self.levels.update(None, &id, level).await?;

        Ok(InventoryLevelDTO {
            available_quantity: saved.stocked_quantity - saved.reserved_quantity,
            id: Id::from_string(saved.id.to_string()).unwrap_or_default(),
            inventory_item_id: saved.inventory_item_id,
            location_id: saved.location_id,
            stocked_quantity: saved.stocked_quantity,
            reserved_quantity: saved.reserved_quantity,
        })
    }

    /// Open reservations, optionally narrowed to one inventory item.
    pub async fn list_reservations(
        &self,
        item_id: Option<&str>,
    ) -> Result<Vec<ReservationItemDTO>> {
        let mut filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        if let Some(item_id) = item_id {
            filter = filter.with_predicate("inventory_item_id", FilterOp::Eq(item_id.to_string()));
        }

        Ok(self
            .reservations
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|r| ReservationItemDTO {
                id: Id::from_string(r.id.to_string()).unwrap_or_default(),
                line_item_id: r.line_item_id,
                inventory_item_id: r.inventory_item_id,
                location_id: r.location_id,
                quantity: r.quantity,
            })
            .collect())
    }

    /// Partial update for an inventory item.
    pub async fn update_item(
        &self,
        id: &str,
        input: UpdateInventoryItemInput,
    ) -> Result<InventoryItemDTO> {
        let mut item = self.items.retrieve(None, id).await?;
        if let Some(sku) = input.sku {
            item.sku = Some(sku);
        }
        if let Some(requires_shipping) = input.requires_shipping {
            item.requires_shipping = requires_shipping;
        }
        item.updated_at = Utc::now();

        let updated = self.items.update(None, id, item).await?;
        Ok(InventoryItemDTO {
            id: Id::from_string(updated.id.to_string()).unwrap_or_default(),
            sku: updated.sku,
            requires_shipping: updated.requires_shipping,
            created_at: updated.created_at,
        })
    }

    /// Soft-deletes an inventory item and every stock level under it.
    ///
    /// The levels go too, in the same transaction. A level whose item is gone is a row
    /// that reports stock for something that no longer exists, and `list_levels` filters
    /// on `deleted_at`, not on the parent.
    pub async fn delete_item(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        let items = self.items.clone();
        let levels = self.levels.clone();
        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let filter = FilterSpec::new()
                    .with_predicate("inventory_item_id", FilterOp::Eq(id.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
                let existing = levels.list(None, &filter).await?;

                with_transaction(conn, move |mut c| {
                    Box::pin(async move {
                        for level in existing {
                            levels
                                .soft_delete(Some(&mut c), &level.id.to_string())
                                .await?;
                        }
                        items.soft_delete(Some(&mut c), &id).await?;
                        Ok(())
                    })
                })
                .await
            })
        })
        .await
    }

    /// Soft-deletes one location's stock level for an item.
    pub async fn delete_level(&self, item_id: &str, location_id: &str) -> Result<()> {
        let filter = FilterSpec::new()
            .with_predicate("inventory_item_id", FilterOp::Eq(item_id.to_string()))
            .with_predicate("location_id", FilterOp::Eq(location_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let levels = self.levels.list(None, &filter).await?;
        let level = levels.into_iter().next().ok_or(Error::NotFound {
            entity: "inventory_level",
            id: format!("{item_id}@{location_id}"),
        })?;

        self.levels.soft_delete(None, &level.id.to_string()).await?;
        Ok(())
    }

    /// Partial update for a reservation.
    ///
    /// A quantity change moves `reserved_quantity` on the level by the delta, in the same
    /// transaction as the row write. `reserved_quantity` is denormalised onto
    /// `inventory_level` and is what `available_quantity` is computed from, so writing the
    /// reservation row alone would leave the level reporting stock that is still held —
    /// an oversell, and silent.
    pub async fn update_reservation(
        &self,
        id: &str,
        input: UpdateReservationInput,
    ) -> Result<ReservationItemDTO> {
        let id = id.to_string();
        let reservations = self.reservations.clone();

        tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                let mut reservation = reservations.retrieve(None, &id).await?;

                let delta = match input.quantity {
                    Some(q) if q < 0 => {
                        return Err(Error::Invalid {
                            code: "invalid_quantity",
                            message: "quantity cannot be negative".to_string(),
                            field: Some("quantity".to_string()),
                        })
                    }
                    Some(q) => q - reservation.quantity,
                    None => 0,
                };

                if let Some(q) = input.quantity {
                    reservation.quantity = q;
                }
                if let Some(description) = input.description {
                    reservation.description = Some(description);
                }
                reservation.updated_at = Utc::now();

                let item_id = reservation.inventory_item_id.clone();
                let loc_id = reservation.location_id.clone();

                with_transaction(conn, move |mut c| {
                    Box::pin(async move {
                        if delta != 0 {
                            sql_query(
                                "UPDATE inventory.inventory_level                                  SET reserved_quantity = GREATEST(0, reserved_quantity + $1), updated_at = now()                                  WHERE inventory_item_id = $2 AND location_id = $3",
                            )
                            .bind::<diesel::sql_types::Integer, _>(delta)
                            .bind::<diesel::sql_types::Text, _>(&item_id)
                            .bind::<diesel::sql_types::Text, _>(&loc_id)
                            .execute(&mut c)
                            .await
                            .map_err(|e| Error::Database(e.to_string()))?;
                        }

                        let updated =
                            reservations.update(Some(&mut c), &id, reservation).await?;
                        Ok(ReservationItemDTO {
                            id: Id::from_string(updated.id.to_string()).unwrap_or_default(),
                            line_item_id: updated.line_item_id,
                            inventory_item_id: updated.inventory_item_id,
                            location_id: updated.location_id,
                            quantity: updated.quantity,
                        })
                    })
                })
                .await
            })
        })
        .await
    }

    pub async fn get_reservation(&self, id: &str) -> Result<ReservationItemDTO> {
        let r = self.reservations.retrieve(None, id).await?;
        Ok(ReservationItemDTO {
            id: Id::from_string(r.id.to_string()).unwrap_or_default(),
            line_item_id: r.line_item_id,
            inventory_item_id: r.inventory_item_id,
            location_id: r.location_id,
            quantity: r.quantity,
        })
    }

    /// Inventory levels where `stocked_quantity <= threshold`.
    ///
    /// Returns up to `limit` levels ordered by ascending `stocked_quantity` so the most
    /// critical items surface first. Zero is a valid and common threshold — it means "out
    /// of stock". The caller is responsible for choosing a business-appropriate threshold.
    ///
    /// # Edge cases
    /// - Empty catalogue: returns an empty `Vec`, not an error.
    /// - `threshold < 0`: clamped to 0 — negative stock cannot exist.
    /// - `limit = 0`: treated as `MAX_PAGE_LIMIT` to avoid silent empty pages.
    pub async fn list_low_stock_levels(
        &self,
        threshold: i32,
        limit: usize,
    ) -> Result<Vec<InventoryLevelDTO>> {
        let effective_threshold = threshold.max(0);
        let effective_limit = if limit == 0 {
            MAX_PAGE_LIMIT
        } else {
            limit.min(MAX_PAGE_LIMIT)
        };

        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_predicate(
                "stocked_quantity",
                FilterOp::Lte(effective_threshold.to_string()),
            )
            .with_order("stocked_quantity", OrderDirection::Asc)
            .with_pagination(0, effective_limit);

        Ok(self
            .levels
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|l| InventoryLevelDTO {
                available_quantity: l.stocked_quantity - l.reserved_quantity,
                id: Id::from_string(l.id.to_string()).unwrap_or_default(),
                inventory_item_id: l.inventory_item_id,
                location_id: l.location_id,
                stocked_quantity: l.stocked_quantity,
                reserved_quantity: l.reserved_quantity,
            })
            .collect())
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
