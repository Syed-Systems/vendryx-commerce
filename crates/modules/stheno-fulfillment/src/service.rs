//! Fulfillment service implementation (DATA_MODEL §2.6, TASKS P5-07..P5-13, P13-19, P13-20).
#![allow(dead_code)]

use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    Address, Fulfillment, FulfillmentItem, FulfillmentLabel, FulfillmentProviderRequest,
    FulfillmentSet, GeoZone, ServiceZone, ShippingOption, ShippingOptionRule, ShippingOptionType,
    ShippingProfile,
};
use crate::provider::FulfillmentProvider;

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
                if let (Some(target_prov), Some(gz_prov)) = (&ctx.province_code, &gz.province_code)
                {
                    if target_prov.to_lowercase() == gz_prov.to_lowercase()
                        && !eligible_zone_ids.contains(&gz.service_zone_id)
                    {
                        eligible_zone_ids.push(gz.service_zone_id.clone());
                    }
                } else if !eligible_zone_ids.contains(&gz.service_zone_id) {
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
        for rule in rules {
            match rule.attribute.as_str() {
                "is_return" => {
                    if let Some(expected) = rule.value.as_bool() {
                        let actual = ctx.is_return.unwrap_or(false);
                        if rule.operator == "eq" && actual != expected {
                            return false;
                        }
                    }
                }
                "customer_id" => {
                    if let Some(expected_id) = rule.value.as_str() {
                        let actual_id = ctx.customer_id.as_deref().unwrap_or("");
                        if rule.operator == "eq" && actual_id != expected_id {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
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
            return Ok(so
                .data
                .as_ref()
                .and_then(|d| {
                    d.get("amount")
                        .and_then(|a| a.as_str())
                        .and_then(|s| s.parse::<Decimal>().ok())
                })
                .unwrap_or(Decimal::ZERO));
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

        if let Some(req) = self.outbound_requests.list(None, &spec).await?.into_iter().next() {
            if req.request_hash != request_hash {
                return Err(Error::Invalid {
                    code: "idempotency_key_payload_mismatch",
                    message: "Idempotency key reused with a different request payload".to_string(),
                    field: None,
                });
            }
            if req.state == "succeeded" {
                if let Some(resp) = req.response {
                    if let Ok(parsed) = serde_json::from_value::<T>(resp) {
                        return Ok(parsed);
                    }
                }
            }
            // in_flight / unknown deliberately fall through to nothing: creating a second
            // consignment is the failure this table exists to prevent. The row stays
            // undecided until a human or a courier lookup settles it.
            if req.state == "in_flight" || req.state == "unknown" {
                return Err(Error::Conflict {
                    code: "provider_request_unsettled",
                    message: format!(
                        "A previous {operation} for {resource_id} is unsettled. Resolve it                          before retrying, or a duplicate consignment will be created."
                    ),
                });
            }
        }

        let now = Utc::now();
        let mut recorded = self
            .outbound_requests
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
            .await?;

        let result = call_fn().await;

        match &result {
            Ok(output) => {
                recorded.state = "succeeded".to_string();
                recorded.response = serde_json::to_value(output).ok();
            }
            Err(e) => {
                // Ambiguous, not failed. We do not know whether a parcel exists.
                recorded.state = "unknown".to_string();
                recorded.error = Some(serde_json::json!({ "error": format!("{e:?}") }));
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
        let idem_key = format!(
            "create_fulfillment:{}:{}",
            input.provider_id,
            Id::<Fulfillment>::new()
        );
        let payload = serde_json::json!({
            "data": input.data,
            "items": input.items,
            "location_id": input.location_id,
        });
        let data_for_call = input.data.clone();
        let items_for_call = input.items.clone();
        let provider_for_call = provider.clone();
        let response = self
            .idempotent_provider_call(
                &idem_key,
                &input.provider_id,
                "create_fulfillment",
                "fulfillment",
                &idem_key,
                &payload,
                move || async move {
                    provider_for_call
                        .create_fulfillment(data_for_call.as_ref(), &items_for_call, None)
                        .await
                },
            )
            .await?;

        let mut delivery_address_id = None;
        if let Some(addr_input) = input.delivery_address {
            let addr = self.create_address_internal(addr_input).await?;
            delivery_address_id = Some(addr.id.to_string());
        }

        let ful = Fulfillment {
            id: Id::new(),
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

        let ful_id_str = ful.id.to_string();
        let created_ful = self.fulfillments.create(None, ful).await?;

        // Add items
        let mut items = Vec::new();
        for item_input in input.items {
            let fi = FulfillmentItem {
                id: Id::new(),
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

        for l_input in all_labels {
            let fl = FulfillmentLabel {
                id: Id::new(),
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
        let spec = FilterSpec::new().with_predicate("deleted_at", FilterOp::IsNull);
        let list = self.fulfillments.list(None, &spec).await?;
        for ful in list {
            if let Some(data) = &ful.data {
                if data.get("reference").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("merchant_reference").and_then(|v| v.as_str()) == Some(reference)
                    || data.get("tracking_number").and_then(|v| v.as_str()) == Some(reference)
                {
                    return self.get_fulfillment(&ful.id.to_string()).await.map(Some);
                }
            }
        }
        Ok(None)
    }

    // ── Internal Helpers ────────────────────────────────────────────────────

    async fn create_address_internal(&self, input: CreateAddressInput) -> Result<Address> {
        let addr = Address {
            id: Id::new(),
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
