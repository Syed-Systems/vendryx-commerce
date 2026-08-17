//! RegionService domain service, backed by Postgres (P11-17).
//!
//! Reference migration for the P11 playbook: no `RwLock<HashMap>`, no
//! `ends_with` id matching, every read and write through `CrudRepository`.

use chrono::Utc;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, OrderDirection, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Country, Region};

/// Countries per region. A region with more than this is a data error, not a
/// page of results.
const MAX_COUNTRIES_PER_REGION: usize = 100;

#[derive(Clone)]
pub struct RegionService {
    regions: PgCrudRepository<Region>,
    countries: PgCrudRepository<Country>,
}

impl RegionService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            regions: PgCrudRepository::new(pool.clone()),
            countries: PgCrudRepository::new(pool),
        }
    }

    fn region_dto(region: &Region, countries: Vec<Country>) -> RegionDTO {
        RegionDTO {
            id: Id::from_ulid(region.id.ulid()),
            name: region.name.clone(),
            currency_code: region.currency_code.clone(),
            automatic_taxes: region.automatic_taxes,
            countries: countries.iter().map(Self::country_dto).collect(),
            created_at: region.created_at,
        }
    }

    fn country_dto(c: &Country) -> CountryDTO {
        CountryDTO {
            // Same ULID, different marker type: the DTO carries the identical
            // identifier, so it is rebuilt rather than reparsed from a string.
            id: Id::from_ulid(c.id.ulid()),
            iso_2: c.iso_2.clone(),
            name: c.name.clone(),
            display_name: c.display_name.clone(),
        }
    }

    fn build_country(iso: &str, region_id: &str) -> Country {
        let now = Utc::now();
        Country {
            id: Id::new(),
            iso_2: iso.to_lowercase(),
            iso_3: format!("{}x", iso.to_lowercase()),
            num_code: "840".to_string(),
            name: iso.to_uppercase(),
            display_name: iso.to_uppercase(),
            region_id: Some(region_id.to_string()),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    async fn countries_of(&self, region_id: &str) -> Result<Vec<Country>> {
        self.countries
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("region_id", FilterOp::Eq(region_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_order("iso_2", OrderDirection::Asc)
                    .with_pagination(0, MAX_COUNTRIES_PER_REGION),
            )
            .await
    }

    /// Loads a live region by id, or `NotFound`.
    async fn live_region(&self, id: &str) -> Result<Region> {
        // `retrieve` already excludes soft-deleted rows and matches the id
        // exactly -- the prefix survives now, so no suffix scan is needed.
        self.regions.retrieve(None, id).await
    }

    pub async fn create_region(&self, input: CreateRegionInput) -> Result<RegionDTO> {
        let now = Utc::now();
        let region = Region {
            id: Id::new(),
            name: input.name,
            currency_code: input.currency_code.to_lowercase(),
            automatic_taxes: input.automatic_taxes.unwrap_or(true),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let region = self.regions.create(None, region).await?;
        let region_id = region.id.to_string();

        let countries: Vec<Country> = input
            .countries
            .iter()
            .map(|iso| Self::build_country(iso, &region_id))
            .collect();

        let countries = if countries.is_empty() {
            Vec::new()
        } else {
            self.countries.create_many(None, countries).await?
        };

        Ok(Self::region_dto(&region, countries))
    }

    pub async fn get_region(&self, id: &str) -> Result<RegionDTO> {
        let region = self.live_region(id).await?;
        let countries = self.countries_of(&region.id.to_string()).await?;
        Ok(Self::region_dto(&region, countries))
    }

    pub async fn list_regions(&self) -> Result<Vec<RegionDTO>> {
        let regions = self
            .regions
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_order("created_at", OrderDirection::Asc)
                    .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT),
            )
            .await?;

        let mut out = Vec::with_capacity(regions.len());
        for region in regions {
            let countries = self.countries_of(&region.id.to_string()).await?;
            out.push(Self::region_dto(&region, countries));
        }
        Ok(out)
    }

    pub async fn get_country_by_iso2(&self, iso_2: &str) -> Result<CountryDTO> {
        let found = self
            .countries
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("iso_2", FilterOp::Eq(iso_2.to_lowercase()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1),
            )
            .await?;

        found
            .first()
            .map(Self::country_dto)
            .ok_or_else(|| Error::NotFound {
                entity: "country",
                id: iso_2.to_string(),
            })
    }

    pub async fn update_region(&self, id: &str, input: UpdateRegionInput) -> Result<RegionDTO> {
        let mut region = self.live_region(id).await?;

        if let Some(name) = input.name {
            region.name = name;
        }
        if let Some(cc) = input.currency_code {
            region.currency_code = cc.to_lowercase();
        }
        if let Some(auto_tax) = input.automatic_taxes {
            region.automatic_taxes = auto_tax;
        }
        region.updated_at = Utc::now();

        let region_id = region.id.to_string();
        let region = self.regions.update(None, &region_id, region).await?;

        if let Some(isos) = input.countries {
            // Replace the set. Detaching first keeps the iso_2 unique index
            // satisfied when a country moves between regions.
            for existing in self.countries_of(&region_id).await? {
                self.countries
                    .delete(None, &existing.id.to_string())
                    .await?;
            }
            let replacements: Vec<Country> = isos
                .iter()
                .map(|iso| Self::build_country(iso, &region_id))
                .collect();
            if !replacements.is_empty() {
                self.countries.create_many(None, replacements).await?;
            }
        }

        let countries = self.countries_of(&region_id).await?;
        Ok(Self::region_dto(&region, countries))
    }

    pub async fn delete_region(&self, id: &str) -> Result<()> {
        let region = self.live_region(id).await?;
        self.regions.soft_delete(None, &region.id.to_string()).await
    }
}
