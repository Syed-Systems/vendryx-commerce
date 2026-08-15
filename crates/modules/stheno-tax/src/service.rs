//! Service layer for `stheno-tax` (P3-12 through P3-15).
//! Hierarchical regions, rate resolution, tax-inclusive reverse computation, TaxProvider trait.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{TaxRate, TaxRateRule, TaxRegion};

// ── TaxProvider trait (P3-15) ────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait TaxProviderTrait: Send + Sync {
    async fn get_tax_lines(&self, items: &[TaxContext]) -> Result<Vec<Vec<TaxLine>>>;
}

// ── System provider (built-in rate lookup) ───────────────────────────────────

pub struct SystemTaxProvider {
    service: TaxService,
}

impl SystemTaxProvider {
    pub fn new(service: TaxService) -> Self {
        Self { service }
    }
}

#[async_trait::async_trait]
impl TaxProviderTrait for SystemTaxProvider {
    async fn get_tax_lines(&self, items: &[TaxContext]) -> Result<Vec<Vec<TaxLine>>> {
        let mut results = Vec::with_capacity(items.len());
        for ctx in items {
            let lines = self.service.resolve_tax_lines(ctx).await?;
            results.push(lines);
        }
        Ok(results)
    }
}

// ── TaxService ───────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct TaxService {
    regions: Arc<RwLock<HashMap<String, TaxRegion>>>,
    rates: Arc<RwLock<HashMap<String, TaxRate>>>,
    rules: Arc<RwLock<HashMap<String, Vec<TaxRateRule>>>>, // tax_rate_id -> rules
}

impl TaxService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_tax_region(&self, input: CreateTaxRegionInput) -> Result<TaxRegionDTO> {
        let id: Id<TaxRegion> = Id::new();
        let region = TaxRegion {
            id,
            country_code: input.country_code.to_lowercase(),
            province_code: input.province_code.map(|s| s.to_lowercase()),
            parent_id: input.parent_id,
            provider_id: input.provider_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut map = self.regions.write().unwrap();
        map.insert(region.id.to_string(), region.clone());

        Ok(TaxRegionDTO {
            id: Id::from_string(region.id.to_string()).unwrap_or_default(),
            country_code: region.country_code,
            province_code: region.province_code,
            parent_id: region.parent_id,
            provider_id: region.provider_id,
            created_at: region.created_at,
        })
    }

    pub async fn create_tax_rate(&self, input: CreateTaxRateInput) -> Result<TaxRateDTO> {
        let id: Id<TaxRate> = Id::new();
        let id_str = id.to_string();

        let rate = TaxRate {
            id,
            tax_region_id: input.tax_region_id.clone(),
            rate: input.rate,
            code: input.code.clone(),
            name: input.name.clone(),
            is_default: input.is_default.unwrap_or(false),
            is_combinable: input.is_combinable.unwrap_or(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        if let Some(input_rules) = &input.rules {
            let mut rule_entities = Vec::new();
            for r in input_rules {
                rule_entities.push(TaxRateRule {
                    id: Id::new(),
                    tax_rate_id: id_str.clone(),
                    reference: r.reference.clone(),
                    reference_id: r.reference_id.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    deleted_at: None,
                });
            }
            let mut rules_map = self.rules.write().unwrap();
            rules_map.insert(id_str.clone(), rule_entities);
        }

        let mut map = self.rates.write().unwrap();
        map.insert(id_str, rate.clone());

        Ok(TaxRateDTO {
            id: Id::from_string(rate.id.to_string()).unwrap_or_default(),
            tax_region_id: rate.tax_region_id,
            rate: rate.rate,
            code: rate.code,
            name: rate.name,
            is_default: rate.is_default,
            is_combinable: rate.is_combinable,
            created_at: rate.created_at,
        })
    }

    pub async fn get_tax_region(&self, id: &str) -> Result<TaxRegionDTO> {
        let map = self.regions.read().unwrap();
        let region = map
            .values()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .or_else(|| map.get(id).filter(|r| r.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "tax_region",
                id: id.to_string(),
            })?;

        Ok(TaxRegionDTO {
            id: Id::from_string(region.id.to_string()).unwrap_or_default(),
            country_code: region.country_code.clone(),
            province_code: region.province_code.clone(),
            parent_id: region.parent_id.clone(),
            provider_id: region.provider_id.clone(),
            created_at: region.created_at,
        })
    }

    pub async fn update_tax_region(
        &self,
        id: &str,
        input: UpdateTaxRegionInput,
    ) -> Result<TaxRegionDTO> {
        let reg_id_str = {
            let mut map = self.regions.write().unwrap();
            let region = map
                .values_mut()
                .find(|r| {
                    (r.id.to_string() == id || r.id.to_string().ends_with(id))
                        && r.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "tax_region",
                    id: id.to_string(),
                })?;

            if let Some(c) = input.country_code {
                region.country_code = c.to_lowercase();
            }
            if input.province_code.is_some() {
                region.province_code = input.province_code.map(|s| s.to_lowercase());
            }
            if input.parent_id.is_some() {
                region.parent_id = input.parent_id;
            }
            if input.provider_id.is_some() {
                region.provider_id = input.provider_id;
            }
            region.updated_at = Utc::now();
            region.id.to_string()
        };

        self.get_tax_region(&reg_id_str).await
    }

    pub async fn delete_tax_region(&self, id: &str) -> Result<()> {
        let mut map = self.regions.write().unwrap();
        let region = map
            .values_mut()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "tax_region",
                id: id.to_string(),
            })?;

        region.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn get_tax_rate(&self, id: &str) -> Result<TaxRateDTO> {
        let map = self.rates.read().unwrap();
        let rate = map
            .values()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .or_else(|| map.get(id).filter(|r| r.deleted_at.is_none()))
            .ok_or_else(|| Error::NotFound {
                entity: "tax_rate",
                id: id.to_string(),
            })?;

        Ok(TaxRateDTO {
            id: Id::from_string(rate.id.to_string()).unwrap_or_default(),
            tax_region_id: rate.tax_region_id.clone(),
            rate: rate.rate,
            code: rate.code.clone(),
            name: rate.name.clone(),
            is_default: rate.is_default,
            is_combinable: rate.is_combinable,
            created_at: rate.created_at,
        })
    }

    pub async fn update_tax_rate(&self, id: &str, input: UpdateTaxRateInput) -> Result<TaxRateDTO> {
        let rate_id_str = {
            let mut map = self.rates.write().unwrap();
            let rate = map
                .values_mut()
                .find(|r| {
                    (r.id.to_string() == id || r.id.to_string().ends_with(id))
                        && r.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "tax_rate",
                    id: id.to_string(),
                })?;

            if let Some(r) = input.rate {
                rate.rate = r;
            }
            if let Some(c) = input.code {
                rate.code = c;
            }
            if let Some(n) = input.name {
                rate.name = n;
            }
            if let Some(d) = input.is_default {
                rate.is_default = d;
            }
            if let Some(comb) = input.is_combinable {
                rate.is_combinable = comb;
            }
            rate.updated_at = Utc::now();
            rate.id.to_string()
        };

        self.get_tax_rate(&rate_id_str).await
    }

    pub async fn delete_tax_rate(&self, id: &str) -> Result<()> {
        let mut map = self.rates.write().unwrap();
        let rate = map
            .values_mut()
            .find(|r| {
                (r.id.to_string() == id || r.id.to_string().ends_with(id)) && r.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "tax_rate",
                id: id.to_string(),
            })?;

        rate.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn create_tax_rate_rule(
        &self,
        tax_rate_id: &str,
        input: CreateTaxRateRuleInput,
    ) -> Result<TaxRateRuleDTO> {
        let id: Id<TaxRateRule> = Id::new();
        let rule = TaxRateRule {
            id,
            tax_rate_id: tax_rate_id.to_string(),
            reference: input.reference.clone(),
            reference_id: input.reference_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut rules_map = self.rules.write().unwrap();
        rules_map
            .entry(tax_rate_id.to_string())
            .or_default()
            .push(rule.clone());

        Ok(TaxRateRuleDTO {
            id: Id::from_string(rule.id.to_string()).unwrap_or_default(),
            tax_rate_id: rule.tax_rate_id,
            reference: rule.reference,
            reference_id: rule.reference_id,
        })
    }

    pub async fn delete_tax_rate_rule(&self, tax_rate_id: &str, rule_id: &str) -> Result<()> {
        let mut rules_map = self.rules.write().unwrap();
        if let Some(list) = rules_map.get_mut(tax_rate_id) {
            list.retain(|r| r.id.to_string() != rule_id);
        }
        Ok(())
    }

    /// Rate resolution (P3-13):
    /// 1. Find the most specific region (province > country).
    /// 2. Check for specific rules matching the item (product_id, product_type_id, etc.).
    /// 3. Fall back to the default rate for the region.
    /// 4. If `is_combinable`, return all combinable rates + the matched rate.
    pub async fn resolve_tax_lines(&self, ctx: &TaxContext) -> Result<Vec<TaxLine>> {
        let regions_map = self.regions.read().unwrap();
        let rates_map = self.rates.read().unwrap();
        let rules_map = self.rules.read().unwrap();

        // Step 1: Find the most specific region (province first, then country)
        let mut best_region: Option<&TaxRegion> = None;

        for r in regions_map.values().filter(|r| r.deleted_at.is_none()) {
            if r.country_code == ctx.country_code.to_lowercase() {
                if let Some(prov) = &ctx.province_code {
                    if r.province_code.as_deref() == Some(&prov.to_lowercase()) {
                        best_region = Some(r);
                        break; // Province match is most specific
                    }
                }
                // Country-level match (no province on region)
                if r.province_code.is_none() && best_region.is_none() {
                    best_region = Some(r);
                }
            }
        }

        let region = best_region.ok_or_else(|| Error::NotFound {
            entity: "tax_region",
            id: format!("{}:{:?}", ctx.country_code, ctx.province_code),
        })?;

        let region_id = region.id.to_string();

        // Step 2: Collect all rates for this region
        let region_rates: Vec<&TaxRate> = rates_map
            .values()
            .filter(|r| r.tax_region_id == region_id && r.deleted_at.is_none())
            .collect();

        // Step 3: Find the best matching rate
        let mut matched_rate: Option<&TaxRate> = None;
        let mut combinable_rates: Vec<&TaxRate> = Vec::new();

        for rate in &region_rates {
            let rate_id = rate.id.to_string();

            // Check if rate has rules that match the context
            if let Some(rate_rules) = rules_map.get(&rate_id) {
                let mut any_match = false;
                for rule in rate_rules {
                    let matches = match rule.reference.as_str() {
                        "product_id" => ctx.product_id.as_deref() == Some(&rule.reference_id),
                        "product_type_id" => {
                            ctx.product_type_id.as_deref() == Some(&rule.reference_id)
                        }
                        "shipping_option_id" => {
                            ctx.shipping_option_id.as_deref() == Some(&rule.reference_id)
                        }
                        _ => false,
                    };
                    if matches {
                        any_match = true;
                        break;
                    }
                }
                if any_match {
                    matched_rate = Some(rate);
                }
            } else if rate.is_default && matched_rate.is_none() {
                matched_rate = Some(rate);
            }

            if rate.is_combinable {
                combinable_rates.push(rate);
            }
        }

        // Step 4: Build tax lines
        let mut lines = Vec::new();

        if let Some(rate) = matched_rate {
            lines.push(TaxLine {
                rate: rate.rate,
                code: rate.code.clone(),
                name: rate.name.clone(),
                tax_rate_id: rate.id.to_string(),
                provider_id: region.provider_id.clone(),
            });

            // Add combinable rates that are different from the matched one
            for cr in &combinable_rates {
                if cr.id.to_string() != rate.id.to_string() {
                    lines.push(TaxLine {
                        rate: cr.rate,
                        code: cr.code.clone(),
                        name: cr.name.clone(),
                        tax_rate_id: cr.id.to_string(),
                        provider_id: region.provider_id.clone(),
                    });
                }
            }
        }

        // If no rate matched, check parent region
        if lines.is_empty() {
            if let Some(parent_id) = &region.parent_id {
                let parent_rates: Vec<&TaxRate> = rates_map
                    .values()
                    .filter(|r| {
                        r.tax_region_id == *parent_id && r.deleted_at.is_none() && r.is_default
                    })
                    .collect();

                if let Some(parent_rate) = parent_rates.first() {
                    lines.push(TaxLine {
                        rate: parent_rate.rate,
                        code: parent_rate.code.clone(),
                        name: parent_rate.name.clone(),
                        tax_rate_id: parent_rate.id.to_string(),
                        provider_id: region.provider_id.clone(),
                    });
                }
            }
        }

        Ok(lines)
    }

    /// Tax-inclusive reverse computation (P3-14).
    /// Given a total that INCLUDES tax, compute the exact subtotal and tax.
    /// Formula: subtotal = total / (1 + rate), tax = total - subtotal
    /// Uses exact Decimal division, never float.
    pub fn compute_tax_inclusive_reverse(
        &self,
        total_inclusive: Decimal,
        rate: Decimal,
        precision: u32,
    ) -> TaxInclusiveBreakdown {
        let one = dec!(1);
        let divisor = one + rate;
        // ponytail: Decimal division is exact to 28 significant digits.
        // For any reasonable money amount this is precise.
        let subtotal = (total_inclusive / divisor).round_dp(precision);
        let tax_total = total_inclusive - subtotal;
        TaxInclusiveBreakdown {
            subtotal,
            tax_total,
            total: total_inclusive,
            rate,
        }
    }
}
