//! Fulfillment service implementation (DATA_MODEL §2.6, TASKS P5-07..P5-13, P13-19, P13-20).
#![allow(dead_code)]

use chrono::Utc;
use rust_decimal::Decimal;
use sha2::Digest;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_db::diesel::{
    self,
    sql_types::{Jsonb, Nullable, Text},
    QueryableByName,
};
use stheno_db::diesel_async::RunQueryDsl;
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Id, Result};
use ulid::Ulid;

use crate::dto::*;
use crate::models::{
    Address, Fulfillment, FulfillmentItem, FulfillmentLabel, FulfillmentProviderRequest,
    FulfillmentSet, FulfillmentWebhookEvent, GeoZone, ServiceZone, ShippingOption,
    ShippingOptionRule, ShippingOptionType, ShippingProfile,
};
use crate::provider::FulfillmentProvider;

fn is_provider_secret_key(key: &str) -> bool {
    let canonical = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    canonical.contains("secret")
        || canonical.contains("password")
        || canonical.contains("authorization")
        || canonical.contains("apikey")
        || canonical.ends_with("token")
}

fn redact_provider_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            object.retain(|key, child| {
                let secret = is_provider_secret_key(key);
                if !secret {
                    redact_provider_secrets(child);
                }
                !secret
            });
        }
        serde_json::Value::Array(values) => {
            for child in values {
                redact_provider_secrets(child);
            }
        }
        _ => {}
    }
}

fn validate_reconciliation_text(value: &str, field: &'static str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
        return Err(Error::Invalid {
            code: "fulfillment_reconciliation_input_invalid",
            message: format!("{field} must contain 1–{max} printable characters."),
            field: Some(field.to_string()),
        });
    }
    Ok(())
}

fn validate_reconciliation_url(value: Option<&str>, field: &'static str) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_reconciliation_text(value, field, 2048)?;
    if !value.starts_with("https://") || value.chars().any(char::is_whitespace) {
        return Err(Error::Invalid {
            code: "fulfillment_reconciliation_url_invalid",
            message: format!("{field} must be an HTTPS URL."),
            field: Some(field.to_string()),
        });
    }
    Ok(())
}

#[derive(Clone)]
pub struct FulfillmentService {
    providers: Arc<RwLock<HashMap<String, Arc<dyn FulfillmentProvider>>>>,
    fulfillment_sets: Arc<PgCrudRepository<FulfillmentSet>>,
    service_zones: Arc<PgCrudRepository<ServiceZone>>,
    geo_zones: Arc<PgCrudRepository<GeoZone>>,
    shipping_profiles: Arc<PgCrudRepository<ShippingProfile>>,
    shipping_option_types: Arc<PgCrudRepository<ShippingOptionType>>,
    shipping_options: Arc<PgCrudRepository<ShippingOption>>,
    shipping_option_rules: Arc<PgCrudRepository<ShippingOptionRule>>,
    fulfillments: Arc<PgCrudRepository<Fulfillment>>,
    fulfillment_items: Arc<PgCrudRepository<FulfillmentItem>>,
    fulfillment_labels: Arc<PgCrudRepository<FulfillmentLabel>>,
    outbound_requests: Arc<PgCrudRepository<FulfillmentProviderRequest>>,
    webhook_events: Arc<PgCrudRepository<FulfillmentWebhookEvent>>,
    addresses: Arc<PgCrudRepository<Address>>,
    pool: DbPool,
}

impl FulfillmentService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            fulfillment_sets: Arc::new(PgCrudRepository::new(pool.clone())),
            service_zones: Arc::new(PgCrudRepository::new(pool.clone())),
            geo_zones: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_profiles: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_option_types: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_options: Arc::new(PgCrudRepository::new(pool.clone())),
            shipping_option_rules: Arc::new(PgCrudRepository::new(pool.clone())),
            fulfillments: Arc::new(PgCrudRepository::new(pool.clone())),
            fulfillment_items: Arc::new(PgCrudRepository::new(pool.clone())),
            fulfillment_labels: Arc::new(PgCrudRepository::new(pool.clone())),
            outbound_requests: Arc::new(PgCrudRepository::new(pool.clone())),
            webhook_events: Arc::new(PgCrudRepository::new(pool.clone())),
            addresses: Arc::new(PgCrudRepository::new(pool.clone())),
            pool,
        }
    }

    // ── Provider Registration (P5-11) ───────────────────────────────────────

    pub fn register_provider(&self, provider: Arc<dyn FulfillmentProvider>) {
        let mut map = self.providers.write().unwrap();
        map.insert(provider.identifier().to_string(), provider);
    }

    /// The providers this build actually registered.
    ///
    /// Read from the live registry rather than a hardcoded list, so it cannot claim a
    /// provider that was never wired in — which is exactly what the API used to do.
    pub fn registered_provider_ids(&self) -> Vec<String> {
        let map = self.providers.read().unwrap();
        let mut ids: Vec<String> = map.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn get_provider(&self, id: &str) -> Result<Arc<dyn FulfillmentProvider>> {
        let map = self.providers.read().unwrap();
        map.get(id).cloned().ok_or_else(|| Error::NotFound {
            entity: "fulfillment_provider",
            id: id.to_string(),
        })
    }

    // ── 1. Fulfillment Sets & Service Zones (P5-08) ─────────────────────────

    pub async fn create_fulfillment_set(
        &self,
        input: CreateFulfillmentSetInput,
    ) -> Result<FulfillmentSetDTO> {
        let set = FulfillmentSet {
            id: Id::new(),
            name: input.name,
            r#type: input.r#type,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.fulfillment_sets.create(None, set).await?;
        Ok(Self::to_set_dto(&created, vec![]))
    }

    pub async fn get_fulfillment_set(&self, id: &str) -> Result<FulfillmentSetDTO> {
        let set = self.fulfillment_sets.retrieve(None, id).await?;
        let zones = self.list_service_zones_for_set(id).await?;
        Ok(Self::to_set_dto(&set, zones))
    }

    pub async fn create_service_zone(
        &self,
        input: CreateServiceZoneInput,
    ) -> Result<ServiceZoneDTO> {
        let zone = ServiceZone {
            id: Id::new(),
            fulfillment_set_id: input.fulfillment_set_id,
            name: input.name,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.service_zones.create(None, zone).await?;
        Ok(Self::to_zone_dto(&created, vec![], vec![]))
    }

    pub async fn create_geo_zone(&self, input: CreateGeoZoneInput) -> Result<GeoZoneDTO> {
        let gz = GeoZone {
            id: Id::new(),
            service_zone_id: input.service_zone_id,
            r#type: input.r#type,
            country_code: input.country_code.to_lowercase(),
            province_code: input.province_code.map(|p| p.to_lowercase()),
            city: input.city,
            postal_expression: input.postal_expression,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.geo_zones.create(None, gz).await?;
        Ok(Self::to_geo_zone_dto(&created))
    }

    async fn list_service_zones_for_set(&self, set_id: &str) -> Result<Vec<ServiceZoneDTO>> {
        let zones = self
            .service_zones
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("fulfillment_set_id", FilterOp::Eq(set_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut res = Vec::new();
        for z in zones {
            let zone_id = z.id.to_string();
            let geo_zones = self
                .geo_zones
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("service_zone_id", FilterOp::Eq(zone_id.clone()))
                        .with_predicate("deleted_at", FilterOp::IsNull)
                        .with_pagination(0, 100),
                )
                .await?
                .into_iter()
                .map(|gz| Self::to_geo_zone_dto(&gz))
                .collect();
            let shipping_options = self.list_shipping_options_for_zone(&zone_id).await?;
            res.push(Self::to_zone_dto(&z, geo_zones, shipping_options));
        }

        Ok(res)
    }

    // ── 2. Shipping Profiles, Types, Options & Rules (P5-09) ─────────────────

    pub async fn create_shipping_profile(
        &self,
        input: CreateShippingProfileInput,
    ) -> Result<ShippingProfileDTO> {
        let sp = ShippingProfile {
            id: Id::new(),
            name: input.name,
            r#type: input.r#type.unwrap_or_else(|| "default".to_string()),
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.shipping_profiles.create(None, sp).await?;
        Ok(Self::to_profile_dto(&created))
    }

    // ── Reads for the admin's fulfillment settings screens ──────────────────
    //
    // The service could create shipping profiles, fulfillment sets, service zones and
    // option types since P5-08, but could not list or fetch any of them. The API
    // therefore answered `/admin/shipping-profiles` with one fabricated row
    // (`sp_default_shipping_profile`) and the rest with empty arrays, so every screen
    // under Settings → Locations was reading invented data.

    pub async fn list_shipping_profiles(&self) -> Result<Vec<ShippingProfileDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        Ok(self
            .shipping_profiles
            .list(None, &filter)
            .await?
            .iter()
            .map(Self::to_profile_dto)
            .collect())
    }

    pub async fn get_shipping_profile(&self, id: &str) -> Result<ShippingProfileDTO> {
        Ok(Self::to_profile_dto(
            &self.shipping_profiles.retrieve(None, id).await?,
        ))
    }

    /// Partial update for a shipping profile. Absent means unchanged.
    pub async fn update_shipping_profile(
        &self,
        id: &str,
        input: UpdateShippingProfileInput,
    ) -> Result<ShippingProfileDTO> {
        let mut sp = self.shipping_profiles.retrieve(None, id).await?;

        if let Some(name) = input.name {
            sp.name = name;
        }
        if let Some(kind) = input.r#type {
            sp.r#type = kind;
        }
        if let Some(metadata) = input.metadata {
            sp.metadata = Some(metadata);
        }
        sp.updated_at = Utc::now();

        let updated = self.shipping_profiles.update(None, id, sp).await?;
        Ok(Self::to_profile_dto(&updated))
    }

    /// Soft-deletes a fulfillment set.
    ///
    /// Service zones are left in place rather than cascaded: they are addressable in their
    /// own right and a cascade here would delete rows no caller asked about. If orphaned
    /// zones become a problem, that is a cleanup job, not a side effect of this call.
    pub async fn delete_fulfillment_set(&self, id: &str) -> Result<()> {
        self.fulfillment_sets.soft_delete(None, id).await?;
        Ok(())
    }

    pub async fn delete_shipping_profile(&self, id: &str) -> Result<()> {
        self.shipping_profiles.soft_delete(None, id).await?;
        Ok(())
    }

    pub async fn list_fulfillment_sets(&self) -> Result<Vec<FulfillmentSetDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        let sets = self.fulfillment_sets.list(None, &filter).await?;

        let mut out = Vec::with_capacity(sets.len());
        for set in sets {
            // Reuses the existing single-set read so the nested service zones and geo
            // zones are assembled exactly one way.
            out.push(self.get_fulfillment_set(&set.id.to_string()).await?);
        }
        Ok(out)
    }

    pub async fn list_service_zones(
        &self,
        fulfillment_set_id: &str,
    ) -> Result<Vec<ServiceZoneDTO>> {
        let set = self.get_fulfillment_set(fulfillment_set_id).await?;
        Ok(set.service_zones)
    }

    pub async fn delete_service_zone(&self, id: &str) -> Result<()> {
        self.service_zones.soft_delete(None, id).await?;
        Ok(())
    }

    pub async fn list_geo_zones(&self) -> Result<Vec<GeoZoneDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        Ok(self
            .geo_zones
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|z| Self::to_geo_zone_dto(&z))
            .collect())
    }

    pub async fn batch_shipping_option_rules(
        &self,
        shipping_option_id: &str,
        create: Vec<CreateShippingOptionRuleInput>,
        delete: Vec<String>,
    ) -> Result<()> {
        for del_id in delete {
            let _ = self.shipping_option_rules.delete(None, &del_id).await;
        }
        for c in create {
            let sor = ShippingOptionRule {
                id: Id::new(),
                shipping_option_id: shipping_option_id.to_string(),
                attribute: c.attribute,
                operator: c.operator,
                value: c.value,
                metadata: c.metadata,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let _ = self.shipping_option_rules.create(None, sor).await;
        }
        Ok(())
    }

    pub async fn list_shipping_option_types(&self) -> Result<Vec<ShippingOptionTypeDTO>> {
        let filter = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT);
        Ok(self
            .shipping_option_types
            .list(None, &filter)
            .await?
            .into_iter()
            .map(|t| ShippingOptionTypeDTO {
                id: t.id.to_string(),
                label: t.label,
                description: t.description,
                code: t.code,
                metadata: t.metadata,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect())
    }

    pub async fn create_shipping_option_type(
        &self,
        input: CreateShippingOptionTypeInput,
    ) -> Result<ShippingOptionTypeDTO> {
        let sot = ShippingOptionType {
            id: Id::new(),
            label: input.label,
            description: input.description,
            code: input.code,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.shipping_option_types.create(None, sot).await?;
        Ok(Self::to_type_dto(&created))
    }

    pub async fn create_shipping_option(
        &self,
        input: CreateShippingOptionInput,
    ) -> Result<ShippingOptionDTO> {
        let so = ShippingOption {
            id: Id::new(),
            name: input.name,
            price_type: input.price_type,
            service_zone_id: input.service_zone_id,
            shipping_profile_id: input.shipping_profile_id,
            provider_id: input.provider_id,
            shipping_option_type_id: input.shipping_option_type_id,
            data: input.data,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.shipping_options.create(None, so).await?;
        Ok(Self::to_shipping_option_dto(&created, vec![]))
    }

    pub async fn add_shipping_option_rule(
        &self,
        input: CreateShippingOptionRuleInput,
    ) -> Result<ShippingOptionRuleDTO> {
        let rule = ShippingOptionRule {
            id: Id::new(),
            shipping_option_id: input.shipping_option_id,
            attribute: input.attribute,
            operator: input.operator,
            value: input.value,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.shipping_option_rules.create(None, rule).await?;
        Ok(Self::to_rule_dto(&created))
    }

    /// One shipping option.
    ///
    /// Only list endpoints existed, so attaching an option to a cart had no way to read
    /// back what it was — which is why the cart named every delivery "Standard Shipping".
    pub async fn get_shipping_option(&self, id: &str) -> Result<ShippingOptionDTO> {
        let option = self.shipping_options.retrieve(None, id).await?;
        let rules = self
            .shipping_option_rules
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("shipping_option_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?
            .into_iter()
            .map(|r| Self::to_rule_dto(&r))
            .collect();
        Ok(Self::to_shipping_option_dto(&option, rules))
    }

    async fn list_shipping_options_for_zone(
        &self,
        zone_id: &str,
    ) -> Result<Vec<ShippingOptionDTO>> {
        let options = self
            .shipping_options
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("service_zone_id", FilterOp::Eq(zone_id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?;

        let mut res = Vec::new();
        for so in options {
            let so_id = so.id.to_string();
            let rules = self
                .shipping_option_rules
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("shipping_option_id", FilterOp::Eq(so_id.clone()))
                        .with_predicate("deleted_at", FilterOp::IsNull)
                        .with_pagination(0, 100),
                )
                .await?
                .into_iter()
                .map(|r| Self::to_rule_dto(&r))
                .collect();
            res.push(Self::to_shipping_option_dto(&so, rules));
        }

        Ok(res)
    }

    pub async fn list_shipping_options(
        &self,
        filter: ShippingOptionFilter,
    ) -> Result<(Vec<ShippingOptionDTO>, usize)> {
        let mut spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        if let Some(zid) = filter.service_zone_id {
            spec = spec.with_predicate("service_zone_id", FilterOp::Eq(zid));
        }
        if let Some(pid) = filter.shipping_profile_id {
            spec = spec.with_predicate("shipping_profile_id", FilterOp::Eq(pid));
        }

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(20);
        spec = spec.with_pagination(offset, limit);

        let (options, total) = self.shipping_options.list_and_count(None, &spec).await?;

        let mut dtos = Vec::new();
        for so in options {
            let so_id = so.id.to_string();
            let rules = self
                .shipping_option_rules
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("shipping_option_id", FilterOp::Eq(so_id.clone()))
                        .with_predicate("deleted_at", FilterOp::IsNull)
                        .with_pagination(0, 100),
                )
                .await?
                .into_iter()
                .map(|r| Self::to_rule_dto(&r))
                .collect();
            dtos.push(Self::to_shipping_option_dto(&so, rules));
        }

        Ok((dtos, total as usize))
    }

    pub async fn update_shipping_option(
        &self,
        id: &str,
        name: Option<String>,
        price_type: Option<String>,
    ) -> Result<ShippingOptionDTO> {
        let mut opt = self.shipping_options.retrieve(None, id).await?;
        if let Some(n) = name {
            opt.name = n;
        }
        if let Some(pt) = price_type {
            opt.price_type = pt;
        }
        opt.updated_at = Utc::now();
        let updated = self.shipping_options.update(None, id, opt).await?;
        let rules = self
            .shipping_option_rules
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("shipping_option_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?
            .into_iter()
            .map(|r| Self::to_rule_dto(&r))
            .collect();
        Ok(Self::to_shipping_option_dto(&updated, rules))
    }

    pub async fn delete_shipping_option(&self, id: &str) -> Result<()> {
        self.shipping_options.soft_delete(None, id).await
    }

    pub async fn get_shipping_option_type(&self, id: &str) -> Result<ShippingOptionTypeDTO> {
        let t = self.shipping_option_types.retrieve(None, id).await?;
        Ok(Self::to_type_dto(&t))
    }

    pub async fn update_shipping_option_type(
        &self,
        id: &str,
        label: Option<String>,
        description: Option<String>,
        code: Option<String>,
    ) -> Result<ShippingOptionTypeDTO> {
        let mut t = self.shipping_option_types.retrieve(None, id).await?;
        if let Some(l) = label {
            t.label = l;
        }
        if let Some(d) = description {
            t.description = Some(d);
        }
        if let Some(c) = code {
            t.code = c;
        }
        t.updated_at = Utc::now();
        let updated = self.shipping_option_types.update(None, id, t).await?;
        Ok(Self::to_type_dto(&updated))
    }

    pub async fn delete_shipping_option_type(&self, id: &str) -> Result<()> {
        self.shipping_option_types.soft_delete(None, id).await
    }

    // ── 3. Eligible Option Resolution (P5-10) ────────────────────────────────

    pub async fn resolve_eligible_shipping_options(
        &self,
        ctx: &EligibilityContext,
    ) -> Result<Vec<ShippingOptionDTO>> {
        let mut eligible_zone_ids = Vec::new();

        // 1. Match Geo Zones
        if let Some(target_country) = &ctx.country_code {
            let target_country = target_country.to_lowercase();
            let geo_list = self
                .geo_zones
                .list(
                    None,
                    &FilterSpec::new()
                        .with_predicate("country_code", FilterOp::Eq(target_country.clone()))
                        .with_predicate("deleted_at", FilterOp::IsNull)
                        .with_pagination(0, 100),
                )
                .await?;

            for gz in geo_list {
                if Self::geo_zone_matches(&gz, ctx)
                    && !eligible_zone_ids.contains(&gz.service_zone_id)
                {
                    eligible_zone_ids.push(gz.service_zone_id.clone());
                }
            }
        }

        // 2. Fetch shipping options for eligible zones
        let mut matched_options = Vec::new();
        for zone_id in eligible_zone_ids {
            let options = self.list_shipping_options_for_zone(&zone_id).await?;
            for opt in options {
                // 3. Evaluate Shipping Option Rules
                let is_eligible = self.evaluate_shipping_option_rules(&opt.rules, ctx);
                if is_eligible {
                    matched_options.push(opt);
                }
            }
        }

        Ok(matched_options)
    }

    fn evaluate_shipping_option_rules(
        &self,
        rules: &[ShippingOptionRuleDTO],
        ctx: &EligibilityContext,
    ) -> bool {
        rules
            .iter()
            .all(|rule| Self::shipping_rule_matches(rule, ctx))
    }

    fn geo_zone_matches(zone: &GeoZone, ctx: &EligibilityContext) -> bool {
        let Some(country) = ctx.country_code.as_deref() else {
            return false;
        };
        if !zone.country_code.eq_ignore_ascii_case(country) {
            return false;
        }
        if zone.province_code.as_deref().is_some_and(|expected| {
            !ctx.province_code
                .as_deref()
                .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
        }) {
            return false;
        }

        match zone.r#type.as_str() {
            "country" => zone.province_code.is_none(),
            "province" => zone
                .province_code
                .as_deref()
                .zip(ctx.province_code.as_deref())
                .is_some_and(|(expected, actual)| expected.eq_ignore_ascii_case(actual)),
            "city" => zone
                .city
                .as_deref()
                .zip(ctx.city.as_deref())
                .is_some_and(|(expected, actual)| expected.eq_ignore_ascii_case(actual)),
            "zip" => zone
                .postal_expression
                .as_ref()
                .and_then(|expression| expression.get("pattern"))
                .and_then(serde_json::Value::as_str)
                .zip(ctx.postal_code.as_deref())
                .is_some_and(|(pattern, postal_code)| {
                    Self::postal_pattern_matches(pattern, postal_code)
                }),
            _ => false,
        }
    }

    fn postal_pattern_matches(pattern: &str, postal_code: &str) -> bool {
        if pattern.is_empty()
            || pattern.chars().count() > 128
            || postal_code.is_empty()
            || postal_code.chars().count() > 32
            || pattern.chars().any(char::is_control)
            || postal_code.chars().any(char::is_control)
        {
            return false;
        }
        let pattern = pattern.to_ascii_lowercase();
        let postal_code = postal_code.to_ascii_lowercase();
        let value = postal_code.chars().collect::<Vec<_>>();
        let mut previous = vec![false; value.len() + 1];
        previous[0] = true;
        for token in pattern.chars() {
            let mut current = vec![false; value.len() + 1];
            if token == '*' {
                current[0] = previous[0];
                for index in 1..=value.len() {
                    current[index] = previous[index] || current[index - 1];
                }
            } else {
                for index in 1..=value.len() {
                    current[index] = previous[index - 1] && token == value[index - 1];
                }
            }
            previous = current;
        }
        previous[value.len()]
    }

    fn shipping_rule_matches(rule: &ShippingOptionRuleDTO, ctx: &EligibilityContext) -> bool {
        if matches!(
            rule.attribute.as_str(),
            "weight" | "total_weight" | "items_count"
        ) {
            let actual = match rule.attribute.as_str() {
                "weight" | "total_weight" => ctx.total_weight,
                "items_count" => ctx.items_count.map(Decimal::from),
                _ => None,
            };
            let Some(actual) = actual else {
                return false;
            };
            let parse_number = |value: &serde_json::Value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| value.as_number().map(ToString::to_string))
                    .and_then(|value| value.parse::<Decimal>().ok())
            };
            return match rule.operator.as_str() {
                "eq" => parse_number(&rule.value).is_some_and(|expected| actual == expected),
                "ne" => parse_number(&rule.value).is_some_and(|expected| actual != expected),
                "gt" => parse_number(&rule.value).is_some_and(|expected| actual > expected),
                "gte" => parse_number(&rule.value).is_some_and(|expected| actual >= expected),
                "lt" => parse_number(&rule.value).is_some_and(|expected| actual < expected),
                "lte" => parse_number(&rule.value).is_some_and(|expected| actual <= expected),
                "in" => rule.value.as_array().is_some_and(|values| {
                    values
                        .iter()
                        .any(|value| parse_number(value).is_some_and(|expected| actual == expected))
                }),
                _ => false,
            };
        }
        let actual = match rule.attribute.as_str() {
            "is_return" => Some(ctx.is_return.unwrap_or(false).to_string()),
            "customer_id" => ctx.customer_id.clone(),
            "customer_group_id" => ctx.customer_group_id.clone(),
            "currency_code" => ctx.currency_code.clone(),
            "sales_channel_id" => ctx.sales_channel_id.clone(),
            "country_code" => ctx.country_code.clone(),
            "province_code" => ctx.province_code.clone(),
            "city" => ctx.city.clone(),
            "postal_code" => ctx.postal_code.clone(),
            _ => None,
        };
        let Some(actual) = actual else {
            return false;
        };
        match rule.operator.as_str() {
            "eq" => {
                rule.value
                    .as_str()
                    .is_some_and(|expected| actual.eq_ignore_ascii_case(expected))
                    || rule
                        .value
                        .as_bool()
                        .is_some_and(|expected| actual == expected.to_string())
            }
            "ne" => {
                rule.value
                    .as_str()
                    .is_some_and(|expected| !actual.eq_ignore_ascii_case(expected))
                    || rule
                        .value
                        .as_bool()
                        .is_some_and(|expected| actual != expected.to_string())
            }
            "in" => rule.value.as_array().is_some_and(|values| {
                values.iter().any(|value| {
                    value
                        .as_str()
                        .is_some_and(|expected| actual.eq_ignore_ascii_case(expected))
                })
            }),
            _ => false,
        }
    }

    // ── 4. Calculated Shipping Pricing (P5-12) ───────────────────────────────

    pub async fn calculate_shipping_price(
        &self,
        shipping_option_id: &str,
        calc_ctx: Option<&CalculatePriceContext>,
    ) -> Result<Decimal> {
        let so = self
            .shipping_options
            .retrieve(None, shipping_option_id)
            .await?;

        if so.price_type == "flat" {
            let amount = so
                .data
                .as_ref()
                .and_then(|data| data.get("amount").or_else(|| data.get("rate")))
                .and_then(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .or_else(|| value.as_number().map(ToString::to_string))
                })
                .and_then(|value| value.parse::<Decimal>().ok())
                .ok_or_else(|| Error::Invalid {
                    code: "invalid_flat_shipping_amount",
                    message: "Flat shipping option requires a valid amount or rate.".to_string(),
                    field: Some("amount".to_string()),
                })?;
            if amount.is_sign_negative() {
                return Err(Error::Invalid {
                    code: "invalid_flat_shipping_amount",
                    message: "Flat shipping amount cannot be negative.".to_string(),
                    field: Some("amount".to_string()),
                });
            }
            return Ok(amount);
        }

        // Calculated pricing via Provider (P5-12)
        let provider = self.get_provider(&so.provider_id)?;
        provider
            .calculate_price(so.data.as_ref(), None, calc_ctx)
            .await
    }

    // ── Outbound request ledger (mirrors payment.payment_provider_request) ───
    //
    // A courier call that times out is worse than a gateway call that times out. Retry a
    // create and you get a second consignment: two parcels, two deliveries, and in a COD
    // market two cash collections against one order. So the durable record is written
    // *before* the call, and a repeat of the same key returns the first answer instead of
    // creating anything.

    #[allow(clippy::too_many_arguments)]
    async fn idempotent_provider_call<F, Fut, T>(
        &self,
        idempotency_key: &str,
        provider_id: &str,
        operation: &str,
        resource_type: &str,
        resource_id: &str,
        payload: &serde_json::Value,
        call_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    {
        // Same canonicalisation as payment's ledger. Duplicated deliberately rather
        // than shared: a module may not depend on another module (check-module-isolation).
        let request_hash = {
            use sha2::{Digest, Sha256};
            let canonical = serde_json::to_string(payload).unwrap_or_default();
            let mut hasher = Sha256::new();
            hasher.update(canonical.as_bytes());
            hex::encode(hasher.finalize())
        };

        let spec = FilterSpec::new()
            .with_predicate("idempotency_key", FilterOp::Eq(idempotency_key.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let mut retried_reconciled_request = None;
        if let Some(mut req) = self
            .outbound_requests
            .list(None, &spec)
            .await?
            .into_iter()
            .next()
        {
            if req.request_hash != request_hash {
                return Err(Error::Invalid {
                    code: "idempotency_key_payload_mismatch",
                    message: "Idempotency key reused with a different request payload".to_string(),
                    field: None,
                });
            }
            if req.state == "succeeded" {
                if let Some(resp) = req.response.as_ref() {
                    if let Ok(parsed) = serde_json::from_value::<T>(resp.clone()) {
                        return Ok(parsed);
                    }
                }
            }
            // in_flight / unknown deliberately fall through to nothing: creating a second
            // consignment is the failure this table exists to prevent.
            if req.state == "in_flight" || req.state == "unknown" {
                return Err(Error::Conflict {
                    code: "provider_request_unsettled",
                    message: format!(
                        "A previous {operation} for {resource_id} is unsettled. Resolve it before retrying, or a duplicate consignment will be created."
                    ),
                });
            }
            if req.state == "failed" {
                #[derive(QueryableByName)]
                struct ClaimedRequest {
                    #[diesel(sql_type = Text)]
                    id: String,
                }
                let request_id = req.id.to_string();
                let claim = stheno_db::tenant::with_tenant_conn(&self.pool, move |conn| {
                    Box::pin(async move {
                        diesel::sql_query(
                            "UPDATE fulfillment.fulfillment_provider_request \
                             SET state = 'in_flight', attempts = attempts + 1, settled_at = NULL \
                             WHERE id = $1 AND state = 'failed' AND deleted_at IS NULL \
                             RETURNING id",
                        )
                        .bind::<Text, _>(&request_id)
                        .load::<ClaimedRequest>(conn)
                        .await
                        .map(|rows| rows.into_iter().next())
                        .map_err(|error| Error::Database(error.to_string()))
                    })
                })
                .await?;
                if claim.is_none() {
                    return Err(Error::Conflict {
                        code: "provider_request_unsettled",
                        message: "The reconciled courier request was claimed by another attempt. Reconcile it before retrying.".to_string(),
                    });
                }
                req.state = "in_flight".to_string();
                req.attempts = req.attempts.saturating_add(1);
                req.settled_at = None;
                retried_reconciled_request = Some(req);
            } else if req.state != "succeeded" {
                return Err(Error::Conflict {
                    code: "provider_request_unsettled",
                    message: format!(
                        "A previous {operation} for {resource_id} is not in a retryable state."
                    ),
                });
            }
        }

        let now = Utc::now();
        let mut recorded = if let Some(request) = retried_reconciled_request {
            request
        } else {
            self.outbound_requests
                .create(
                    None,
                    FulfillmentProviderRequest {
                        id: Id::new(),
                        idempotency_key: idempotency_key.to_string(),
                        provider_id: provider_id.to_string(),
                        operation: operation.to_string(),
                        resource_type: resource_type.to_string(),
                        resource_id: resource_id.to_string(),
                        request_hash,
                        state: "in_flight".to_string(),
                        response: None,
                        error: None,
                        attempts: 1,
                        first_sent_at: now,
                        settled_at: None,
                        deleted_at: None,
                    },
                )
                .await?
        };

        let result = call_fn().await;

        match &result {
            Ok(output) => {
                recorded.state = "succeeded".to_string();
                recorded.response = serde_json::to_value(output).ok();
            }
            Err(e) => {
                // Ambiguous, not failed. We do not know whether a parcel exists.
                recorded.state = "unknown".to_string();
                if recorded.error.is_none() {
                    recorded.error = Some(serde_json::json!({ "error": format!("{e:?}") }));
                }
            }
        }
        recorded.settled_at = Some(Utc::now());
        let id = recorded.id.to_string();
        let _ = self.outbound_requests.update(None, &id, recorded).await;

        result
    }

    // ── 5. Fulfillment Lifecycle (P5-13) ─────────────────────────────────────

    pub async fn create_fulfillment(
        &self,
        input: CreateFulfillmentInput,
    ) -> Result<FulfillmentDTO> {
        self.create_fulfillment_with_key(input, None).await
    }

    /// Creates a fulfillment with a caller-supplied stable idempotency key.
    ///
    /// Use an order-derived key for human-confirmed social dispatch. Replays return the
    /// first fulfillment record; an unresolved provider call remains blocked by the
    /// outbound request ledger.
    pub async fn create_fulfillment_idempotent(
        &self,
        input: CreateFulfillmentInput,
        idempotency_key: &str,
    ) -> Result<FulfillmentDTO> {
        let idempotency_key = idempotency_key.trim();
        if idempotency_key.is_empty()
            || idempotency_key.len() > 256
            || idempotency_key.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "fulfillment_idempotency_key_invalid",
                message: "Fulfillment idempotency key must be 1–256 printable characters."
                    .to_string(),
                field: Some("idempotency_key".to_string()),
            });
        }
        self.create_fulfillment_with_key(input, Some(idempotency_key))
            .await
    }

    /// Creates a fulfillment while supplying per-tenant provider secrets only to the
    /// outbound call. These values are deliberately excluded from persisted fulfillment
    /// data and the durable request ledger payload.
    pub async fn create_fulfillment_idempotent_with_provider_data(
        &self,
        input: CreateFulfillmentInput,
        idempotency_key: &str,
        provider_data: serde_json::Value,
    ) -> Result<FulfillmentDTO> {
        let idempotency_key = idempotency_key.trim();
        if idempotency_key.is_empty()
            || idempotency_key.len() > 256
            || idempotency_key.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "fulfillment_idempotency_key_invalid",
                message: "Fulfillment idempotency key must be 1–256 printable characters."
                    .to_string(),
                field: Some("idempotency_key".to_string()),
            });
        }
        self.create_fulfillment_with_key_and_provider_data(
            input,
            Some(idempotency_key),
            Some(provider_data),
        )
        .await
    }

    async fn create_fulfillment_with_key(
        &self,
        input: CreateFulfillmentInput,
        stable_key: Option<&str>,
    ) -> Result<FulfillmentDTO> {
        self.create_fulfillment_with_key_and_provider_data(input, stable_key, None)
            .await
    }

    async fn create_fulfillment_with_key_and_provider_data(
        &self,
        input: CreateFulfillmentInput,
        stable_key: Option<&str>,
        provider_data: Option<serde_json::Value>,
    ) -> Result<FulfillmentDTO> {
        let mut line_item_ids = std::collections::BTreeSet::new();
        for item in &input.items {
            if item.line_item_id.trim().is_empty() || item.quantity <= 0 {
                return Err(Error::Invalid {
                    code: "fulfillment_item_invalid",
                    message: "Fulfillment items require a line item ID and positive quantity."
                        .to_string(),
                    field: Some("items".to_string()),
                });
            }
            if !line_item_ids.insert(item.line_item_id.trim()) {
                return Err(Error::Invalid {
                    code: "fulfillment_item_duplicate",
                    message: "A fulfillment cannot contain the same order line more than once."
                        .to_string(),
                    field: Some("items".to_string()),
                });
            }
        }
        let provider = self.get_provider(&input.provider_id)?;

        // Keyed on the caller's reference when there is one. Without a stable reference
        // a retry cannot be told apart from a second, intentional shipment, so the
        // caller must supply one to get dedupe rather than merely a durable record.
        //
        // P37-26: the no-reference case used to be the constant "no_reference", which did
        // not withhold dedupe — it made every unreferenced call *share* one key. The
        // second genuinely different parcel for a courier was then refused as a replay
        // (`idempotency_key_payload_mismatch`), and shipping two parcels through one
        // courier is the ordinary case, not a retry. A fresh id withholds dedupe, which is
        // what the paragraph above always meant, and still writes the durable ledger row.
        //
        // The `shipping_option_id` branch had that same defect and outlived the fix: a
        // shipping option is a rate, not a consignment. Every parcel sent by one method
        // reuses it, so an outbound shipment and the return that follows it collided on
        // one key and the second was refused — deterministically, for the ordinary case.
        // `CreateFulfillmentInput` carries no idempotency key, so there is no caller
        // reference to key on at all: withhold dedupe here too rather than invent one out
        // of a field that does not identify the request.
        // A caller-supplied key is tenant-scoped independently of provider. The provider
        // ID is part of the payload hash, so editing the courier after an ambiguous call
        // cannot create a second consignment under a different provider namespace.
        let (fulfillment_id, key_suffix) = match stable_key {
            Some(stable_key) => {
                let tenant_id = stheno_db::tenant::current()?;
                let mut hasher = sha2::Sha256::new();
                hasher.update(tenant_id.as_bytes());
                hasher.update([0]);
                hasher.update(stable_key.as_bytes());
                let digest = hasher.finalize();
                let mut ulid_bytes = [0_u8; 16];
                ulid_bytes.copy_from_slice(&digest[..16]);
                let id = Id::<Fulfillment>::from_ulid(Ulid::from(u128::from_be_bytes(ulid_bytes)));
                (id, stable_key.to_string())
            }
            None => {
                let id = Id::<Fulfillment>::new();
                (id, id.to_string())
            }
        };
        let idem_key = match stable_key {
            Some(_) => format!("create_fulfillment:{key_suffix}"),
            None => format!("create_fulfillment:{}:{key_suffix}", input.provider_id),
        };
        let mut provider_payload_hash = provider_data
            .clone()
            .unwrap_or_else(|| input.data.clone().unwrap_or(serde_json::Value::Null));
        redact_provider_secrets(&mut provider_payload_hash);
        let mut input_data_hash = input.data.clone().unwrap_or(serde_json::Value::Null);
        redact_provider_secrets(&mut input_data_hash);
        let payload = serde_json::json!({
            "provider_id": input.provider_id,
            "data": input_data_hash,
            "provider_request": provider_payload_hash,
            "items": input.items,
            "location_id": input.location_id,
            "shipping_option_id": input.shipping_option_id,
            "delivery_address": input.delivery_address,
            "labels": input.labels,
            "metadata": input.metadata,
        });
        let data_for_call = provider_data.or_else(|| input.data.clone());
        let items_for_call = input.items.clone();
        let provider_for_call = provider.clone();
        let ful_id_str = fulfillment_id.to_string();
        let response = self
            .idempotent_provider_call(
                &idem_key,
                &input.provider_id,
                "create_fulfillment",
                "fulfillment",
                &ful_id_str,
                &payload,
                move || async move {
                    let mut response = provider_for_call
                        .create_fulfillment(data_for_call.as_ref(), &items_for_call, None)
                        .await?;
                    if let Some(data) = &mut response.data {
                        redact_provider_secrets(data);
                    }
                    for label in &mut response.labels {
                        if let Some(metadata) = &mut label.metadata {
                            redact_provider_secrets(metadata);
                        }
                    }
                    Ok(response)
                },
            )
            .await?;

        match self.get_fulfillment(&ful_id_str).await {
            Ok(existing) => {
                let labels = if !response.labels.is_empty() {
                    response.labels.as_slice()
                } else {
                    input.labels.as_deref().unwrap_or_default()
                };
                self.ensure_fulfillment_children(&existing, &input.items, labels)
                    .await?;
                return self.get_fulfillment(&ful_id_str).await;
            }
            Err(Error::NotFound { .. }) => {}
            Err(error) => return Err(error),
        }

        let mut delivery_address_id = None;
        if let Some(addr_input) = input.delivery_address {
            let address_id = Id::<Address>::from_ulid(fulfillment_id.ulid());
            let addr = self
                .create_address_internal_with_id(addr_input, address_id)
                .await?;
            delivery_address_id = Some(addr.id.to_string());
        }

        let ful = Fulfillment {
            id: fulfillment_id,
            location_id: input.location_id,
            provider_id: input.provider_id,
            shipping_option_id: input.shipping_option_id,
            delivery_address_id,
            packed_at: None,
            shipped_at: None,
            delivered_at: None,
            canceled_at: None,
            data: response.data.or(input.data),
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created_ful = self.fulfillments.create(None, ful).await?;

        // Add items
        let mut items = Vec::new();
        for item_input in input.items {
            let fi = FulfillmentItem {
                id: Self::child_id::<FulfillmentItem>(&fulfillment_id, &item_input.line_item_id),
                fulfillment_id: ful_id_str.clone(),
                title: item_input.title,
                sku: item_input.sku,
                barcode: item_input.barcode,
                line_item_id: item_input.line_item_id,
                inventory_item_id: item_input.inventory_item_id,
                quantity: item_input.quantity,
                metadata: item_input.metadata,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let created_item = self.fulfillment_items.create(None, fi).await?;
            items.push(Self::to_item_dto(&created_item));
        }

        // Add labels
        let mut labels = Vec::new();
        let all_labels = if !response.labels.is_empty() {
            response.labels
        } else {
            input.labels.unwrap_or_default()
        };

        let mut known_label_values = std::collections::BTreeSet::new();
        for (index, l_input) in all_labels
            .into_iter()
            .filter(|label| {
                known_label_values.insert((
                    label.tracking_number.clone(),
                    label.tracking_url.clone(),
                    label.label_url.clone(),
                ))
            })
            .enumerate()
        {
            let fl = FulfillmentLabel {
                id: Self::child_id::<FulfillmentLabel>(
                    &fulfillment_id,
                    &format!("label:{index}:{}", l_input.tracking_number),
                ),
                fulfillment_id: ful_id_str.clone(),
                tracking_number: l_input.tracking_number,
                tracking_url: l_input.tracking_url,
                label_url: l_input.label_url,
                metadata: l_input.metadata,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            let created_label = self.fulfillment_labels.create(None, fl).await?;
            labels.push(Self::to_label_dto(&created_label));
        }

        let address = if let Some(addr_id) = &created_ful.delivery_address_id {
            self.addresses
                .retrieve(None, addr_id)
                .await
                .ok()
                .map(|a| Self::to_address_dto(&a))
        } else {
            None
        };

        Ok(Self::to_fulfillment_dto(
            &created_ful,
            address,
            items,
            labels,
        ))
    }

    pub async fn mark_as_shipped(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = self.fulfillments.retrieve(None, id).await?;
        ful.shipped_at = Some(Utc::now());
        ful.updated_at = Utc::now();
        self.fulfillments.update(None, id, ful).await?;
        self.get_fulfillment(id).await
    }

    pub async fn mark_as_delivered(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = self.fulfillments.retrieve(None, id).await?;
        ful.delivered_at = Some(Utc::now());
        ful.updated_at = Utc::now();
        self.fulfillments.update(None, id, ful).await?;
        self.get_fulfillment(id).await
    }

    pub async fn cancel_fulfillment(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = self.fulfillments.retrieve(None, id).await?;

        if ful.canceled_at.is_some() {
            return Err(Error::Conflict {
                code: "fulfillment_already_canceled",
                message: "Fulfillment is already canceled".to_string(),
            });
        }

        let provider = self.get_provider(&ful.provider_id)?;
        provider.cancel_fulfillment(ful.data.as_ref()).await?;

        ful.canceled_at = Some(Utc::now());
        ful.updated_at = Utc::now();
        self.fulfillments.update(None, id, ful).await?;
        self.get_fulfillment(id).await
    }

    pub async fn get_fulfillment(&self, id: &str) -> Result<FulfillmentDTO> {
        let ful = self.fulfillments.retrieve(None, id).await?;

        let items = self
            .fulfillment_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("fulfillment_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?
            .into_iter()
            .map(|i| Self::to_item_dto(&i))
            .collect();

        let labels = self
            .fulfillment_labels
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate("fulfillment_id", FilterOp::Eq(id.to_string()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 100),
            )
            .await?
            .into_iter()
            .map(|l| Self::to_label_dto(&l))
            .collect();

        let address = if let Some(addr_id) = &ful.delivery_address_id {
            self.addresses
                .retrieve(None, addr_id)
                .await
                .ok()
                .map(|a| Self::to_address_dto(&a))
        } else {
            None
        };

        Ok(Self::to_fulfillment_dto(&ful, address, items, labels))
    }

    pub async fn lookup_by_reference(&self, reference: &str) -> Result<Option<FulfillmentDTO>> {
        if reference.trim().is_empty()
            || reference.len() > 256
            || reference.chars().any(char::is_control)
        {
            return Err(Error::Invalid {
                code: "fulfillment_reference_invalid",
                message: "Fulfillment reference must be 1–256 printable characters.".to_string(),
                field: Some("reference".to_string()),
            });
        }
        #[derive(QueryableByName)]
        struct FulfillmentIdRow {
            #[diesel(sql_type = Text)]
            id: String,
        }
        let reference = reference.to_string();
        let reference_for_sql = reference.clone();
        let found = stheno_db::tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "SELECT id FROM fulfillment.fulfillment \
                      WHERE deleted_at IS NULL AND data ->> 'reference' = $1 \
                      ORDER BY created_at DESC LIMIT 1",
                )
                .bind::<Text, _>(&reference_for_sql)
                .load::<FulfillmentIdRow>(conn)
                .await
                .map(|rows| rows.into_iter().next().map(|row| row.id))
                .map_err(|error| Error::Database(error.to_string()))
            })
        })
        .await?;
        if let Some(id) = found {
            return self.get_fulfillment(&id).await.map(Some);
        }

        // Keep legacy merchant-reference and tracking-number lookups working while
        // staff-confirmed social orders use the indexed stable `reference` field above.
        let spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        let list = self.fulfillments.list(None, &spec).await?;
        for ful in list {
            if let Some(data) = &ful.data {
                if data.get("reference").and_then(|v| v.as_str()) == Some(reference.as_str())
                    || data.get("merchant_reference").and_then(|v| v.as_str())
                        == Some(reference.as_str())
                    || data.get("tracking_number").and_then(|v| v.as_str())
                        == Some(reference.as_str())
                {
                    return self.get_fulfillment(&ful.id.to_string()).await.map(Some);
                }
            }
        }
        Ok(None)
    }

    // ── Internal Helpers ────────────────────────────────────────────────────

    fn child_id<T: stheno_types::Prefixed>(fulfillment_id: &Id<Fulfillment>, key: &str) -> Id<T> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(fulfillment_id.ulid().to_string().as_bytes());
        hasher.update([0]);
        hasher.update(key.as_bytes());
        let digest = hasher.finalize();
        let mut bytes = [0_u8; 16];
        bytes.copy_from_slice(&digest[..16]);
        Id::from_ulid(Ulid::from(u128::from_be_bytes(bytes)))
    }

    async fn ensure_fulfillment_children(
        &self,
        fulfillment: &FulfillmentDTO,
        expected_items: &[CreateFulfillmentItemInput],
        expected_labels: &[CreateFulfillmentLabelInput],
    ) -> Result<()> {
        let fulfillment_id = Id::<Fulfillment>::from_string(&fulfillment.id)?;
        let fulfillment_id_str = fulfillment.id.as_str();
        let current_items = self
            .fulfillment_items
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate(
                        "fulfillment_id",
                        FilterOp::Eq(fulfillment_id_str.to_string()),
                    )
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT),
            )
            .await?;
        let mut known_line_items = current_items
            .iter()
            .map(|item| item.line_item_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for expected in expected_items {
            if !known_line_items.insert(expected.line_item_id.clone()) {
                continue;
            }
            self.fulfillment_items
                .create(
                    None,
                    FulfillmentItem {
                        id: Self::child_id::<FulfillmentItem>(
                            &fulfillment_id,
                            &expected.line_item_id,
                        ),
                        fulfillment_id: fulfillment_id_str.to_string(),
                        title: expected.title.clone(),
                        sku: expected.sku.clone(),
                        barcode: expected.barcode.clone(),
                        line_item_id: expected.line_item_id.clone(),
                        inventory_item_id: expected.inventory_item_id.clone(),
                        quantity: expected.quantity,
                        metadata: expected.metadata.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
                    },
                )
                .await?;
        }

        let current_labels = self
            .fulfillment_labels
            .list(
                None,
                &FilterSpec::new()
                    .with_predicate(
                        "fulfillment_id",
                        FilterOp::Eq(fulfillment_id_str.to_string()),
                    )
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, stheno_db::filter::MAX_PAGE_LIMIT),
            )
            .await?;
        let mut known_labels = current_labels
            .iter()
            .map(|label| {
                (
                    label.tracking_number.clone(),
                    label.tracking_url.clone(),
                    label.label_url.clone(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        for (index, expected) in expected_labels.iter().enumerate() {
            if !known_labels.insert((
                expected.tracking_number.clone(),
                expected.tracking_url.clone(),
                expected.label_url.clone(),
            )) {
                continue;
            }
            self.fulfillment_labels
                .create(
                    None,
                    FulfillmentLabel {
                        id: Self::child_id::<FulfillmentLabel>(
                            &fulfillment_id,
                            &format!("label:{index}:{}", expected.tracking_number),
                        ),
                        fulfillment_id: fulfillment_id_str.to_string(),
                        tracking_number: expected.tracking_number.clone(),
                        tracking_url: expected.tracking_url.clone(),
                        label_url: expected.label_url.clone(),
                        metadata: expected.metadata.clone(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        deleted_at: None,
                    },
                )
                .await?;
        }
        Ok(())
    }

    async fn create_address_internal(&self, input: CreateAddressInput) -> Result<Address> {
        self.create_address_internal_with_id(input, Id::new()).await
    }

    async fn create_address_internal_with_id(
        &self,
        input: CreateAddressInput,
        address_id: Id<Address>,
    ) -> Result<Address> {
        let address_id_string = address_id.to_string();
        match self.addresses.retrieve(None, &address_id_string).await {
            Ok(existing) => return Ok(existing),
            Err(Error::NotFound { .. }) => {}
            Err(error) => return Err(error),
        }
        let addr = Address {
            id: address_id,
            customer_id: input.customer_id,
            company: input.company,
            first_name: input.first_name,
            last_name: input.last_name,
            address_1: input.address_1,
            address_2: input.address_2,
            city: input.city,
            country_code: input.country_code.map(|c| c.to_lowercase()),
            province: input.province,
            postal_code: input.postal_code,
            phone: input.phone,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        self.addresses.create(None, addr).await
    }

    fn to_set_dto(s: &FulfillmentSet, zones: Vec<ServiceZoneDTO>) -> FulfillmentSetDTO {
        FulfillmentSetDTO {
            id: s.id.to_string(),
            name: s.name.clone(),
            r#type: s.r#type.clone(),
            service_zones: zones,
            metadata: s.metadata.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }

    fn to_zone_dto(
        z: &ServiceZone,
        geo_zones: Vec<GeoZoneDTO>,
        options: Vec<ShippingOptionDTO>,
    ) -> ServiceZoneDTO {
        ServiceZoneDTO {
            id: z.id.to_string(),
            fulfillment_set_id: z.fulfillment_set_id.clone(),
            name: z.name.clone(),
            geo_zones,
            shipping_options: options,
            metadata: z.metadata.clone(),
            created_at: z.created_at,
            updated_at: z.updated_at,
        }
    }

    fn to_geo_zone_dto(gz: &GeoZone) -> GeoZoneDTO {
        GeoZoneDTO {
            id: gz.id.to_string(),
            service_zone_id: gz.service_zone_id.clone(),
            r#type: gz.r#type.clone(),
            country_code: gz.country_code.clone(),
            province_code: gz.province_code.clone(),
            city: gz.city.clone(),
            postal_expression: gz.postal_expression.clone(),
            metadata: gz.metadata.clone(),
            created_at: gz.created_at,
            updated_at: gz.updated_at,
        }
    }

    fn to_profile_dto(p: &ShippingProfile) -> ShippingProfileDTO {
        ShippingProfileDTO {
            id: p.id.to_string(),
            name: p.name.clone(),
            r#type: p.r#type.clone(),
            metadata: p.metadata.clone(),
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }

    fn to_type_dto(t: &ShippingOptionType) -> ShippingOptionTypeDTO {
        ShippingOptionTypeDTO {
            id: t.id.to_string(),
            label: t.label.clone(),
            description: t.description.clone(),
            code: t.code.clone(),
            metadata: t.metadata.clone(),
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }

    fn to_shipping_option_dto(
        so: &ShippingOption,
        rules: Vec<ShippingOptionRuleDTO>,
    ) -> ShippingOptionDTO {
        ShippingOptionDTO {
            id: so.id.to_string(),
            name: so.name.clone(),
            price_type: so.price_type.clone(),
            service_zone_id: so.service_zone_id.clone(),
            shipping_profile_id: so.shipping_profile_id.clone(),
            provider_id: so.provider_id.clone(),
            shipping_option_type_id: so.shipping_option_type_id.clone(),
            rules,
            data: so.data.clone(),
            metadata: so.metadata.clone(),
            created_at: so.created_at,
            updated_at: so.updated_at,
        }
    }

    fn to_rule_dto(r: &ShippingOptionRule) -> ShippingOptionRuleDTO {
        ShippingOptionRuleDTO {
            id: r.id.to_string(),
            shipping_option_id: r.shipping_option_id.clone(),
            attribute: r.attribute.clone(),
            operator: r.operator.clone(),
            value: r.value.clone(),
            metadata: r.metadata.clone(),
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }

    fn to_fulfillment_dto(
        f: &Fulfillment,
        delivery_address: Option<AddressDTO>,
        items: Vec<FulfillmentItemDTO>,
        labels: Vec<FulfillmentLabelDTO>,
    ) -> FulfillmentDTO {
        FulfillmentDTO {
            id: f.id.to_string(),
            location_id: f.location_id.clone(),
            provider_id: f.provider_id.clone(),
            shipping_option_id: f.shipping_option_id.clone(),
            delivery_address,
            items,
            labels,
            packed_at: f.packed_at,
            shipped_at: f.shipped_at,
            delivered_at: f.delivered_at,
            canceled_at: f.canceled_at,
            data: f.data.clone(),
            metadata: f.metadata.clone(),
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }

    fn to_item_dto(i: &FulfillmentItem) -> FulfillmentItemDTO {
        FulfillmentItemDTO {
            id: i.id.to_string(),
            fulfillment_id: i.fulfillment_id.clone(),
            title: i.title.clone(),
            sku: i.sku.clone(),
            barcode: i.barcode.clone(),
            line_item_id: i.line_item_id.clone(),
            inventory_item_id: i.inventory_item_id.clone(),
            quantity: i.quantity,
            metadata: i.metadata.clone(),
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }

    fn to_label_dto(l: &FulfillmentLabel) -> FulfillmentLabelDTO {
        FulfillmentLabelDTO {
            id: l.id.to_string(),
            fulfillment_id: l.fulfillment_id.clone(),
            tracking_number: l.tracking_number.clone(),
            tracking_url: l.tracking_url.clone(),
            label_url: l.label_url.clone(),
            metadata: l.metadata.clone(),
            created_at: l.created_at,
            updated_at: l.updated_at,
        }
    }

    pub async fn list_provider_requests(
        &self,
        tenant_id: &str,
        limit: usize,
    ) -> Result<Vec<FulfillmentProviderRequestDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, limit);

        stheno_db::tenant::scope(tenant_id, async {
            let rows = self.outbound_requests.list(None, &spec).await?;
            Ok(rows
                .into_iter()
                .map(|r| FulfillmentProviderRequestDTO {
                    id: r.id.to_string(),
                    idempotency_key: r.idempotency_key,
                    provider_id: r.provider_id,
                    operation: r.operation,
                    resource_type: r.resource_type,
                    resource_id: r.resource_id,
                    state: r.state,
                    response: r.response,
                    error: r.error,
                    attempts: r.attempts,
                    first_sent_at: r.first_sent_at,
                    settled_at: r.settled_at,
                })
                .collect())
        })
        .await
    }

    /// Settles an ambiguous social courier request after an operator checks the courier record.
    ///
    /// The tenant-scoped request row is compare-and-swapped from `unknown`, so two operators
    /// cannot settle one provider response differently. A confirmed acceptance stores only
    /// canonical consignment/tracking fields; a confirmed absence leaves a retryable `failed`
    /// row that the next stable-key dispatch claims in place.
    pub async fn reconcile_unknown_social_dispatch(
        &self,
        stable_reference: &str,
        actor_id: &str,
        evidence_reference: &str,
        resolution: SocialDispatchResolution,
    ) -> Result<()> {
        validate_reconciliation_text(stable_reference, "stable_reference", 256)?;
        validate_reconciliation_text(actor_id, "actor_id", 128)?;
        validate_reconciliation_text(evidence_reference, "evidence_reference", 256)?;

        let idempotency_key = format!("create_fulfillment:{stable_reference}");
        let spec = FilterSpec::new()
            .with_predicate("idempotency_key", FilterOp::Eq(idempotency_key))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        let request = self
            .outbound_requests
            .list(None, &spec)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound {
                entity: "unsettled fulfillment request",
                id: stable_reference.to_string(),
            })?;
        if request.operation != "create_fulfillment"
            || request.resource_type != "fulfillment"
            || request.state != "unknown"
        {
            return Err(Error::Conflict {
                code: "provider_request_not_reconcilable",
                message: "Only an unknown social courier creation can be reconciled.".to_string(),
            });
        }

        let (state, response) = match resolution {
            SocialDispatchResolution::Accepted {
                courier_reference,
                tracking_number,
                tracking_url,
                label_url,
            } => {
                validate_reconciliation_text(&courier_reference, "courier_reference", 128)?;
                validate_reconciliation_text(&tracking_number, "tracking_number", 128)?;
                validate_reconciliation_url(tracking_url.as_deref(), "tracking_url")?;
                validate_reconciliation_url(label_url.as_deref(), "label_url")?;
                let response = CreateFulfillmentResponse {
                    data: Some(serde_json::json!({
                        "reference": stable_reference,
                        "consignment_id": courier_reference,
                        "reconciled": true,
                    })),
                    labels: vec![CreateFulfillmentLabelInput {
                        tracking_number,
                        tracking_url,
                        label_url,
                        metadata: None,
                    }],
                };
                (
                    "succeeded",
                    Some(serde_json::to_value(response).map_err(|error| {
                        Error::Internal(
                            format!("Could not encode courier reconciliation: {error}").into(),
                        )
                    })?),
                )
            }
            SocialDispatchResolution::NotCreated => ("failed", None),
        };
        let audit = serde_json::json!({
            "outcome": state,
            "actor_id": actor_id,
            "evidence_reference": evidence_reference,
            "resolved_at": Utc::now(),
        });

        #[derive(QueryableByName)]
        struct UpdatedRequest {
            #[diesel(sql_type = Text)]
            id: String,
        }
        let request_id = request.id.to_string();
        let updated = stheno_db::tenant::with_tenant_conn(&self.pool, move |conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "UPDATE fulfillment.fulfillment_provider_request \
                     SET state = $1, response = $2, error = $3, settled_at = now() \
                     WHERE id = $4 AND state = 'unknown' AND deleted_at IS NULL \
                     RETURNING id",
                )
                .bind::<Text, _>(state)
                .bind::<Nullable<Jsonb>, _>(response)
                .bind::<Jsonb, _>(audit)
                .bind::<Text, _>(&request_id)
                .load::<UpdatedRequest>(conn)
                .await
                .map(|rows| rows.into_iter().next())
                .map_err(|error| Error::Database(error.to_string()))
            })
        })
        .await?;
        if updated.is_none() {
            return Err(Error::Conflict {
                code: "provider_request_already_reconciled",
                message: "The courier request changed while it was being reconciled. Refresh and inspect its current state.".to_string(),
            });
        }
        Ok(())
    }

    // ── Fulfillment Webhook Event Store & Replay Guard (P21-04) ────────────────
    pub async fn receive_webhook(
        &self,
        provider_id: &str,
        raw_body: &[u8],
        _signature: Option<&str>,
    ) -> Result<FulfillmentWebhookEventDTO> {
        let payload: serde_json::Value =
            serde_json::from_slice(raw_body).map_err(|e| Error::Invalid {
                code: "invalid_webhook_payload",
                message: format!("Malformed webhook payload: {e}"),
                field: None,
            })?;

        let event_id = payload
            .get("event_id")
            .or_else(|| payload.get("id"))
            .or_else(|| payload.get("consignment_id"))
            .or_else(|| payload.get("tracking_code"))
            .and_then(|v| v.as_str())
            .unwrap_or("evt_default")
            .to_string();

        let event_type = payload
            .get("event")
            .or_else(|| payload.get("type"))
            .or_else(|| payload.get("status"))
            .and_then(|v| v.as_str())
            .unwrap_or("status_update")
            .to_string();

        let spec = FilterSpec::new()
            .with_predicate("provider_id", FilterOp::Eq(provider_id.to_string()))
            .with_predicate("provider_event_id", FilterOp::Eq(event_id.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let existing = self.webhook_events.list(None, &spec).await?;
        if let Some(event) = existing.into_iter().next() {
            return Ok(FulfillmentWebhookEventDTO {
                id: event.id.to_string(),
                provider_id: event.provider_id,
                provider_event_id: event.provider_event_id,
                event_type: event.event_type,
                payload: event.payload,
                signature_verified: event.signature_verified,
                provider_created_at: event.provider_created_at,
                state: "ignored".to_string(),
                processed_at: event.processed_at,
                error: event.error,
                created_at: event.created_at,
            });
        }

        let event = FulfillmentWebhookEvent {
            id: Id::new(),
            provider_id: provider_id.to_string(),
            provider_event_id: event_id,
            event_type,
            payload,
            signature_verified: true,
            provider_created_at: Utc::now(),
            state: "received".to_string(),
            processed_at: None,
            error: None,
            created_at: Utc::now(),
            deleted_at: None,
        };

        let created = self.webhook_events.create(None, event).await?;

        // Shared status reducer: if provider is registered and maps webhook, apply status
        if let Ok(provider) = self.get_provider(provider_id) {
            if let Ok(Some((consignment_ref, courier_status))) =
                provider.map_webhook_event(&created.payload)
            {
                let _ = self
                    .apply_status_update(&consignment_ref, provider_id, courier_status.as_str())
                    .await;
            }
        }

        Ok(FulfillmentWebhookEventDTO {
            id: created.id.to_string(),
            provider_id: created.provider_id,
            provider_event_id: created.provider_event_id,
            event_type: created.event_type,
            payload: created.payload,
            signature_verified: created.signature_verified,
            provider_created_at: created.provider_created_at,
            state: created.state,
            processed_at: created.processed_at,
            error: created.error,
            created_at: created.created_at,
        })
    }

    // ── Shared Courier Status Reducer & Polling Fallback (P23-10, P23-11) ─────

    /// Apply a courier status update to a fulfillment record.
    ///
    /// This is the SINGLE shared status reducer in the engine:
    /// - Webhooks and polling both invoke this method (P23-10).
    /// - If status is Delivered, sets `delivered_at`.
    /// - If status is InTransit/PickedUp/OutForDelivery and `shipped_at` is None, sets `shipped_at`.
    /// - If status is Cancelled and `canceled_at` is None, sets `canceled_at`.
    /// - If status is `Unknown(raw)`, leaves fulfillment in that named state and raises an alert (P23-11).
    pub async fn apply_status_update(
        &self,
        consignment_ref: &str,
        provider_id: &str,
        raw_status: &str,
    ) -> Result<Option<FulfillmentDTO>> {
        let status = crate::status::CourierStatus::parse(raw_status);

        let Some(mut ful) = self.find_fulfillment_by_reference(consignment_ref).await? else {
            tracing::warn!(
                consignment_ref = %consignment_ref,
                provider_id = %provider_id,
                "Received courier status update for unknown consignment reference"
            );
            return Ok(None);
        };

        let now = Utc::now();
        let mut data = ful.data.clone().unwrap_or_else(|| serde_json::json!({}));
        if !data.is_object() {
            data = serde_json::json!({});
        }

        data["courier_status"] = serde_json::json!(status.as_str());
        data["courier_raw_status"] = serde_json::json!(raw_status);
        data["status_updated_at"] = serde_json::json!(now);

        if status.is_unknown() {
            data["unknown_status_alert"] = serde_json::json!(true);
            data["alert_reason"] = serde_json::json!(format!(
                "Unrecognized courier status '{raw_status}' from provider '{provider_id}'"
            ));
            tracing::warn!(
                fulfillment_id = %ful.id,
                provider_id = %provider_id,
                raw_status = %raw_status,
                "Unknown courier status alert: fulfillment left in named unknown state"
            );
        }

        if status.is_delivered() && ful.delivered_at.is_none() {
            ful.delivered_at = Some(now);
        } else if matches!(
            status,
            crate::status::CourierStatus::PickedUp
                | crate::status::CourierStatus::InTransit
                | crate::status::CourierStatus::OutForDelivery
        ) && ful.shipped_at.is_none()
        {
            ful.shipped_at = Some(now);
        } else if matches!(status, crate::status::CourierStatus::Cancelled)
            && ful.canceled_at.is_none()
        {
            ful.canceled_at = Some(now);
        }

        ful.data = Some(data);
        ful.updated_at = now;

        let ful_id = ful.id.to_string();
        self.fulfillments.update(None, &ful_id, ful).await?;

        self.get_fulfillment(&ful_id).await.map(Some)
    }

    /// Internal helper to find fulfillment by various reference fields.
    pub(crate) async fn find_fulfillment_by_reference(
        &self,
        reference: &str,
    ) -> Result<Option<Fulfillment>> {
        // Check by ID directly
        if let Ok(ful) = self.fulfillments.retrieve(None, reference).await {
            return Ok(Some(ful));
        }

        // Check by label tracking_number
        let label_spec = FilterSpec::new()
            .with_predicate("tracking_number", FilterOp::Eq(reference.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        if let Some(label) = self
            .fulfillment_labels
            .list(None, &label_spec)
            .await?
            .into_iter()
            .next()
        {
            if let Ok(ful) = self
                .fulfillments
                .retrieve(None, &label.fulfillment_id)
                .await
            {
                return Ok(Some(ful));
            }
        }

        // Scan fulfillments data field
        let spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        let list = self.fulfillments.list(None, &spec).await?;
        for ful in list {
            if let Some(data) = &ful.data {
                if data.get("reference").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("merchant_reference").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("tracking_number").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("consignment_id").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("consignment_cid").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("tracking_code").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("tracking_id").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("parcel_id").and_then(|v| v.as_str()) == Some(reference)
                {
                    return Ok(Some(ful));
                }
            }
        }

        Ok(None)
    }

    /// Polling fallback for tracking status (P23-10).
    ///
    /// Queries provider tracking and passes result through `apply_status_update`
    /// (the same reducer used by webhooks - zero duplicate code path).
    pub async fn poll_consignment_status(
        &self,
        provider_id: &str,
        reference: &str,
    ) -> Result<Option<FulfillmentDTO>> {
        let provider = self.get_provider(provider_id)?;
        let status = provider.get_tracking_status(reference).await?;
        self.apply_status_update(reference, provider_id, status.as_str())
            .await
    }

    fn to_address_dto(a: &Address) -> AddressDTO {
        AddressDTO {
            id: a.id.to_string(),
            customer_id: a.customer_id.clone(),
            company: a.company.clone(),
            first_name: a.first_name.clone(),
            last_name: a.last_name.clone(),
            address_1: a.address_1.clone(),
            address_2: a.address_2.clone(),
            city: a.city.clone(),
            country_code: a.country_code.clone(),
            province: a.province.clone(),
            postal_code: a.postal_code.clone(),
            phone: a.phone.clone(),
            metadata: a.metadata.clone(),
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}
