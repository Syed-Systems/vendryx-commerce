//! RegionService domain service.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{Country, Region};

#[derive(Clone, Default)]
pub struct RegionService {
    regions: Arc<RwLock<HashMap<String, Region>>>,
    countries: Arc<RwLock<HashMap<String, Vec<Country>>>>, // region_id -> countries
}

impl RegionService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_region(&self, input: CreateRegionInput) -> Result<RegionDTO> {
        let region_id = Id::new();
        let region = Region {
            id: region_id,
            name: input.name,
            currency_code: input.currency_code.to_lowercase(),
            automatic_taxes: input.automatic_taxes.unwrap_or(true),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut country_dtos = Vec::new();
        let mut country_entities = Vec::new();

        for iso in input.countries {
            let cid = Id::new();
            let c = Country {
                id: cid,
                iso_2: iso.to_lowercase(),
                iso_3: format!("{}x", iso.to_lowercase()),
                num_code: "840".to_string(),
                name: iso.to_uppercase(),
                display_name: iso.to_uppercase(),
                region_id: Some(region.id.to_string()),
            };
            country_dtos.push(CountryDTO {
                id: Id::from_string(c.id.to_string()).unwrap_or_default(),
                iso_2: c.iso_2.clone(),
                name: c.name.clone(),
                display_name: c.display_name.clone(),
            });
            country_entities.push(c);
        }

        let mut reg_map = self.regions.write().unwrap();
        reg_map.insert(region.id.to_string(), region.clone());

        let mut country_map = self.countries.write().unwrap();
        country_map.insert(region.id.to_string(), country_entities);

        Ok(RegionDTO {
            id: Id::from_string(region.id.to_string()).unwrap_or_default(),
            name: region.name,
            currency_code: region.currency_code,
            automatic_taxes: region.automatic_taxes,
            countries: country_dtos,
            created_at: region.created_at,
        })
    }

    pub async fn get_region(&self, id: &str) -> Result<RegionDTO> {
        let reg_map = self.regions.read().unwrap();
        let region = reg_map
            .values()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .or_else(|| reg_map.get(id).filter(|r| r.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "region",
                id: id.to_string(),
            })?;

        let country_map = self.countries.read().unwrap();
        let countries = country_map
            .get(&region.id.to_string())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|c| CountryDTO {
                id: Id::from_string(c.id.to_string()).unwrap_or_default(),
                iso_2: c.iso_2,
                name: c.name,
                display_name: c.display_name,
            })
            .collect();

        Ok(RegionDTO {
            id: Id::from_string(region.id.to_string()).unwrap_or_default(),
            name: region.name.clone(),
            currency_code: region.currency_code.clone(),
            automatic_taxes: region.automatic_taxes,
            countries,
            created_at: region.created_at,
        })
    }

    pub async fn list_regions(&self) -> Result<Vec<RegionDTO>> {
        let reg_map = self.regions.read().unwrap();
        let country_map = self.countries.read().unwrap();

        let list = reg_map
            .values()
            .filter(|r| r.deleted_at.is_none())
            .map(|r| {
                let countries = country_map
                    .get(&r.id.to_string())
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| CountryDTO {
                        id: Id::from_string(c.id.to_string()).unwrap_or_default(),
                        iso_2: c.iso_2,
                        name: c.name,
                        display_name: c.display_name,
                    })
                    .collect();

                RegionDTO {
                    id: Id::from_string(r.id.to_string()).unwrap_or_default(),
                    name: r.name.clone(),
                    currency_code: r.currency_code.clone(),
                    automatic_taxes: r.automatic_taxes,
                    countries,
                    created_at: r.created_at,
                }
            })
            .collect();

        Ok(list)
    }

    pub async fn get_country_by_iso2(&self, iso_2: &str) -> Result<CountryDTO> {
        let country_map = self.countries.read().unwrap();
        let iso_lower = iso_2.to_lowercase();
        for list in country_map.values() {
            if let Some(c) = list.iter().find(|c| c.iso_2 == iso_lower) {
                return Ok(CountryDTO {
                    id: Id::from_string(c.id.to_string()).unwrap_or_default(),
                    iso_2: c.iso_2.clone(),
                    name: c.name.clone(),
                    display_name: c.display_name.clone(),
                });
            }
        }
        Err(Error::NotFound {
            entity: "country",
            id: iso_2.to_string(),
        })
    }

    pub async fn update_region(&self, id: &str, input: UpdateRegionInput) -> Result<RegionDTO> {
        let region_id_str = {
            let mut reg_map = self.regions.write().unwrap();
            let region = reg_map
                .values_mut()
                .find(|r| {
                    (r.id.to_string() == id || r.id.to_string().ends_with(id))
                        && r.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "region",
                    id: id.to_string(),
                })?;

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
            let region_id_str = region.id.to_string();

            if let Some(countries) = input.countries {
                let mut country_entities = Vec::new();
                for iso in countries {
                    let cid = Id::new();
                    country_entities.push(Country {
                        id: cid,
                        iso_2: iso.to_lowercase(),
                        iso_3: format!("{}x", iso.to_lowercase()),
                        num_code: "840".to_string(),
                        name: iso.to_uppercase(),
                        display_name: iso.to_uppercase(),
                        region_id: Some(region_id_str.clone()),
                    });
                }
                let mut country_map = self.countries.write().unwrap();
                country_map.insert(region_id_str.clone(), country_entities);
            }

            region_id_str
        };

        self.get_region(&region_id_str).await
    }

    pub async fn delete_region(&self, id: &str) -> Result<()> {
        let mut reg_map = self.regions.write().unwrap();
        let region = reg_map
            .values_mut()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "region",
                id: id.to_string(),
            })?;

        region.deleted_at = Some(Utc::now());
        Ok(())
    }
}
