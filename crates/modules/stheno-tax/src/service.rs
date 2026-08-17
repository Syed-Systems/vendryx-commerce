//! Service layer for `stheno-tax`, backed by Postgres (P12-11).
//! Hierarchical regions, rate resolution, tax-inclusive reverse computation, TaxProvider trait.

use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
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

#[derive(Clone)]
pub struct TaxService {
    regions: PgCrudRepository<TaxRegion>,
    rates: PgCrudRepository<TaxRate>,
    rules: PgCrudRepository<TaxRateRule>,
}

impl TaxService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            regions: PgCrudRepository::new(pool.clone()),
            rates: PgCrudRepository::new(pool.clone()),
            rules: PgCrudRepository::new(pool),
        }
    }

    pub async fn create_tax_region(&self, input: CreateTaxRegionInput) -> Result<TaxRegionDTO> {
        let id: Id<TaxRegion> = Id::new();
        let now = Utc::now();
        let region = TaxRegion {
            id,
            country_code: input.country_code.to_lowercase(),
            province_code: input.province_code.map(|s| s.to_lowercase()),
            parent_id: input.parent_id,
            provider_id: input.provider_id,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.regions.create(None, region).await?;

        Ok(TaxRegionDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            country_code: created.country_code,
            province_code: created.province_code,
            parent_id: created.parent_id,
            provider_id: created.provider_id,
            created_at: created.created_at,
        })
    }

    pub async fn create_tax_rate(&self, input: CreateTaxRateInput) -> Result<TaxRateDTO> {
        let id: Id<TaxRate> = Id::new();
        let id_str = id.to_string();
        let now = Utc::now();

        let rate = TaxRate {
            id,
            tax_region_id: input.tax_region_id.clone(),
            rate: input.rate,
            code: input.code.clone(),
            name: input.name.clone(),
            is_default: input.is_default.unwrap_or(false),
            is_combinable: input.is_combinable.unwrap_or(false),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.rates.create(None, rate).await?;

        if let Some(input_rules) = &input.rules {
            let mut rule_entities = Vec::new();
            for r in input_rules {
                rule_entities.push(TaxRateRule {
                    id: Id::new(),
                    tax_rate_id: id_str.clone(),
                    reference: r.reference.clone(),
                    reference_id: r.reference_id.clone(),
                    created_at: now,
                    updated_at: now,
                    deleted_at: None,
                });
            }
            if !rule_entities.is_empty() {
                self.rules.create_many(None, rule_entities).await?;
            }
        }

        Ok(TaxRateDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            tax_region_id: created.tax_region_id,
            rate: created.rate,
            code: created.code,
            name: created.name,
            is_default: created.is_default,
            is_combinable: created.is_combinable,
            created_at: created.created_at,
        })
    }

    pub async fn get_tax_region(&self, id: &str) -> Result<TaxRegionDTO> {
        let region = self.regions.retrieve(None, id).await?;

        Ok(TaxRegionDTO {
            id: Id::from_string(region.id.to_string()).unwrap_or_default(),
            country_code: region.country_code,
            province_code: region.province_code,
            parent_id: region.parent_id,
            provider_id: region.provider_id,
            created_at: region.created_at,
        })
    }

    pub async fn update_tax_region(
        &self,
        id: &str,
        input: UpdateTaxRegionInput,
    ) -> Result<TaxRegionDTO> {
        let mut region = self.regions.retrieve(None, id).await?;

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

        let updated = self
            .regions
            .update(None, &region.id.to_string(), region)
            .await?;

        Ok(TaxRegionDTO {
            id: Id::from_string(updated.id.to_string()).unwrap_or_default(),
            country_code: updated.country_code,
            province_code: updated.province_code,
            parent_id: updated.parent_id,
            provider_id: updated.provider_id,
            created_at: updated.created_at,
        })
    }

    pub async fn delete_tax_region(&self, id: &str) -> Result<()> {
        let region = self.regions.retrieve(None, id).await?;
        self.regions.soft_delete(None, &region.id.to_string()).await
    }

    pub async fn get_tax_rate(&self, id: &str) -> Result<TaxRateDTO> {
        let rate = self.rates.retrieve(None, id).await?;

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

    pub async fn update_tax_rate(&self, id: &str, input: UpdateTaxRateInput) -> Result<TaxRateDTO> {
        let mut rate = self.rates.retrieve(None, id).await?;

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

        let updated = self.rates.update(None, &rate.id.to_string(), rate).await?;

        Ok(TaxRateDTO {
            id: Id::from_string(updated.id.to_string()).unwrap_or_default(),
            tax_region_id: updated.tax_region_id,
            rate: updated.rate,
            code: updated.code,
            name: updated.name,
            is_default: updated.is_default,
            is_combinable: updated.is_combinable,
            created_at: updated.created_at,
        })
    }

    pub async fn delete_tax_rate(&self, id: &str) -> Result<()> {
        let rate = self.rates.retrieve(None, id).await?;
        self.rates.soft_delete(None, &rate.id.to_string()).await
    }

    pub async fn create_tax_rate_rule(
        &self,
        tax_rate_id: &str,
        input: CreateTaxRateRuleInput,
    ) -> Result<TaxRateRuleDTO> {
        let id: Id<TaxRateRule> = Id::new();
        let now = Utc::now();
        let rule = TaxRateRule {
            id,
            tax_rate_id: tax_rate_id.to_string(),
            reference: input.reference.clone(),
            reference_id: input.reference_id.clone(),
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let created = self.rules.create(None, rule).await?;

        Ok(TaxRateRuleDTO {
            id: Id::from_string(created.id.to_string()).unwrap_or_default(),
            tax_rate_id: created.tax_rate_id,
            reference: created.reference,
            reference_id: created.reference_id,
        })
    }

    pub async fn delete_tax_rate_rule(&self, _tax_rate_id: &str, rule_id: &str) -> Result<()> {
        self.rules.delete(None, rule_id).await
    }

    /// Rate resolution (P3-13, P12-11):
    /// 1. Find the most specific region (province > country).
    /// 2. Check for specific rules matching the item (product_id, product_type_id, etc.).
    /// 3. Fall back to the default rate for the region.
    /// 4. If `is_combinable`, return all combinable rates + the matched rate.
    pub async fn resolve_tax_lines(&self, ctx: &TaxContext) -> Result<Vec<TaxLine>> {
        let country_lower = ctx.country_code.to_lowercase();
        let prov_lower = ctx.province_code.as_ref().map(|s| s.to_lowercase());

        // Find candidate regions
        let filter = FilterSpec::new()
            .with_predicate("country_code", FilterOp::Eq(country_lower.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);

        let candidate_regions = self.regions.list(None, &filter).await?;

        let mut best_region: Option<TaxRegion> = None;
        for r in candidate_regions {
            if let Some(p) = &prov_lower {
                if r.province_code.as_deref() == Some(p) {
                    best_region = Some(r);
                    break;
                }
            }
            if r.province_code.is_none() && best_region.is_none() {
                best_region = Some(r);
            }
        }

        let region = best_region.ok_or_else(|| Error::NotFound {
            entity: "tax_region",
            id: format!("{}:{:?}", ctx.country_code, ctx.province_code),
        })?;

        let region_id = region.id.to_string();

        // Load rates for this region
        let rate_filter = FilterSpec::new()
            .with_predicate("tax_region_id", FilterOp::Eq(region_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);

        let region_rates = self.rates.list(None, &rate_filter).await?;
        let rate_ids: Vec<String> = region_rates.iter().map(|r| r.id.to_string()).collect();

        // Load rules for these rates
        let rules = if !rate_ids.is_empty() {
            let rule_filter = FilterSpec::new()
                .with_predicate("tax_rate_id", FilterOp::In(rate_ids))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 100);
            self.rules.list(None, &rule_filter).await?
        } else {
            Vec::new()
        };

        // Step 3: Find best matching rate
        let mut matched_rate: Option<&TaxRate> = None;
        let mut combinable_rates: Vec<&TaxRate> = Vec::new();

        for rate in &region_rates {
            let rate_id = rate.id.to_string();
            let rate_rules: Vec<&TaxRateRule> =
                rules.iter().filter(|r| r.tax_rate_id == rate_id).collect();

            if !rate_rules.is_empty() {
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
                let parent_rate_filter = FilterSpec::new()
                    .with_predicate("tax_region_id", FilterOp::Eq(parent_id.clone()))
                    .with_predicate("is_default", FilterOp::Eq(true.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 10);

                let parent_rates = self.rates.list(None, &parent_rate_filter).await?;
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
        compute_tax_inclusive_reverse(total_inclusive, rate, precision)
    }
}

/// Standalone Tax-inclusive reverse computation (P3-14).
pub fn compute_tax_inclusive_reverse(
    total_inclusive: Decimal,
    rate: Decimal,
    precision: u32,
) -> TaxInclusiveBreakdown {
    let one = dec!(1);
    let divisor = one + rate;
    let subtotal = (total_inclusive / divisor).round_dp(precision);
    let tax_total = total_inclusive - subtotal;
    TaxInclusiveBreakdown {
        subtotal,
        tax_total,
        total: total_inclusive,
        rate,
    }
}
