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
    Address, Fulfillment, FulfillmentItem, FulfillmentLabel, FulfillmentSet, GeoZone, ServiceZone,
    ShippingOption, ShippingOptionRule, ShippingOptionType, ShippingProfile,
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
            addresses: Arc::new(PgCrudRepository::new(pool.clone())),
            pool,
        }
    }

    // ── Provider Registration (P5-11) ───────────────────────────────────────

    pub fn register_provider(&self, provider: Arc<dyn FulfillmentProvider>) {
        let mut map = self.providers.write().unwrap();
        map.insert(provider.identifier().to_string(), provider);
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

    // ── 5. Fulfillment Lifecycle (P5-13) ─────────────────────────────────────

    pub async fn create_fulfillment(
        &self,
        input: CreateFulfillmentInput,
    ) -> Result<FulfillmentDTO> {
        let provider = self.get_provider(&input.provider_id)?;

        // Call provider.create_fulfillment (P5-13)
        let response = provider
            .create_fulfillment(input.data.as_ref(), &input.items, None)
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
