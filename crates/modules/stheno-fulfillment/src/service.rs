//! Fulfillment service implementation (DATA_MODEL §2.6, TASKS P5-07..P5-13).
#![allow(dead_code)]

use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    Address, Fulfillment, FulfillmentItem, FulfillmentLabel, FulfillmentSet, GeoZone, ServiceZone,
    ShippingOption, ShippingOptionRule, ShippingOptionType, ShippingProfile,
};
use crate::provider::FulfillmentProvider;

#[derive(Clone, Default)]
pub struct FulfillmentService {
    providers: Arc<RwLock<HashMap<String, Arc<dyn FulfillmentProvider>>>>,
    fulfillment_sets: Arc<RwLock<HashMap<String, FulfillmentSet>>>,
    service_zones: Arc<RwLock<HashMap<String, ServiceZone>>>,
    geo_zones: Arc<RwLock<HashMap<String, Vec<GeoZone>>>>, // key: service_zone_id
    shipping_profiles: Arc<RwLock<HashMap<String, ShippingProfile>>>,
    shipping_option_types: Arc<RwLock<HashMap<String, ShippingOptionType>>>,
    shipping_options: Arc<RwLock<HashMap<String, ShippingOption>>>,
    shipping_option_rules: Arc<RwLock<HashMap<String, Vec<ShippingOptionRule>>>>, // key: shipping_option_id
    fulfillments: Arc<RwLock<HashMap<String, Fulfillment>>>,
    fulfillment_items: Arc<RwLock<HashMap<String, Vec<FulfillmentItem>>>>, // key: fulfillment_id
    fulfillment_labels: Arc<RwLock<HashMap<String, Vec<FulfillmentLabel>>>>, // key: fulfillment_id
    addresses: Arc<RwLock<HashMap<String, Address>>>,
}

impl FulfillmentService {
    pub fn new() -> Self {
        Self::default()
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

        let dto = Self::to_set_dto(&set, vec![]);
        {
            let mut map = self.fulfillment_sets.write().unwrap();
            map.insert(set.id.to_string(), set);
        }

        Ok(dto)
    }

    pub async fn get_fulfillment_set(&self, id: &str) -> Result<FulfillmentSetDTO> {
        let set = {
            let map = self.fulfillment_sets.read().unwrap();
            map.get(id).cloned().ok_or_else(|| Error::NotFound {
                entity: "fulfillment_set",
                id: id.to_string(),
            })?
        };

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

        let dto = Self::to_zone_dto(&zone, vec![], vec![]);
        {
            let mut map = self.service_zones.write().unwrap();
            map.insert(zone.id.to_string(), zone);
        }

        Ok(dto)
    }

    pub async fn create_geo_zone(&self, input: CreateGeoZoneInput) -> Result<GeoZoneDTO> {
        let gz = GeoZone {
            id: Id::new(),
            service_zone_id: input.service_zone_id.clone(),
            r#type: input.r#type,
            country_code: input.country_code.to_lowercase(),
            province_code: input.province_code.map(|p| p.to_lowercase()),
            city: input.city,
            postal_expression: input.postal_expression,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = Self::to_geo_zone_dto(&gz);
        {
            let mut map = self.geo_zones.write().unwrap();
            map.entry(input.service_zone_id).or_default().push(gz);
        }

        Ok(dto)
    }

    async fn list_service_zones_for_set(&self, set_id: &str) -> Result<Vec<ServiceZoneDTO>> {
        let zones = {
            let map = self.service_zones.read().unwrap();
            map.values()
                .filter(|z| z.fulfillment_set_id == set_id && z.deleted_at.is_none())
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut res = Vec::new();
        for z in zones {
            let zone_id = z.id.to_string();
            let geo_zones = {
                let map = self.geo_zones.read().unwrap();
                map.get(&zone_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|gz| Self::to_geo_zone_dto(&gz))
                    .collect()
            };
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

        let dto = Self::to_profile_dto(&sp);
        {
            let mut map = self.shipping_profiles.write().unwrap();
            map.insert(sp.id.to_string(), sp);
        }

        Ok(dto)
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

        let dto = Self::to_type_dto(&sot);
        {
            let mut map = self.shipping_option_types.write().unwrap();
            map.insert(sot.id.to_string(), sot);
        }

        Ok(dto)
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

        let dto = Self::to_shipping_option_dto(&so, vec![]);
        {
            let mut map = self.shipping_options.write().unwrap();
            map.insert(so.id.to_string(), so);
        }

        Ok(dto)
    }

    pub async fn add_shipping_option_rule(
        &self,
        input: CreateShippingOptionRuleInput,
    ) -> Result<ShippingOptionRuleDTO> {
        let rule = ShippingOptionRule {
            id: Id::new(),
            shipping_option_id: input.shipping_option_id.clone(),
            attribute: input.attribute,
            operator: input.operator,
            value: input.value,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = Self::to_rule_dto(&rule);
        {
            let mut map = self.shipping_option_rules.write().unwrap();
            map.entry(input.shipping_option_id).or_default().push(rule);
        }

        Ok(dto)
    }

    async fn list_shipping_options_for_zone(
        &self,
        zone_id: &str,
    ) -> Result<Vec<ShippingOptionDTO>> {
        let options = {
            let map = self.shipping_options.read().unwrap();
            map.values()
                .filter(|so| so.service_zone_id == zone_id && so.deleted_at.is_none())
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut res = Vec::new();
        for so in options {
            let so_id = so.id.to_string();
            let rules = {
                let map = self.shipping_option_rules.read().unwrap();
                map.get(&so_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| Self::to_rule_dto(&r))
                    .collect()
            };
            res.push(Self::to_shipping_option_dto(&so, rules));
        }

        Ok(res)
    }

    pub async fn list_shipping_options(
        &self,
        filter: ShippingOptionFilter,
    ) -> Result<(Vec<ShippingOptionDTO>, usize)> {
        let options = {
            let map = self.shipping_options.read().unwrap();
            let mut list: Vec<_> = map
                .values()
                .filter(|so| {
                    if so.deleted_at.is_some() {
                        return false;
                    }
                    if let Some(ref zid) = filter.service_zone_id {
                        if &so.service_zone_id != zid {
                            return false;
                        }
                    }
                    if let Some(ref pid) = filter.shipping_profile_id {
                        if &so.shipping_profile_id != pid {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect();
            list.sort_by_key(|b| std::cmp::Reverse(b.created_at));
            list
        };

        let total = options.len();
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(20);

        let paged: Vec<_> = options.into_iter().skip(offset).take(limit).collect();
        let mut dtos = Vec::new();
        for so in paged {
            let so_id = so.id.to_string();
            let rules = {
                let map = self.shipping_option_rules.read().unwrap();
                map.get(&so_id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| Self::to_rule_dto(&r))
                    .collect()
            };
            dtos.push(Self::to_shipping_option_dto(&so, rules));
        }

        Ok((dtos, total))
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
            let gz_map = self.geo_zones.read().unwrap();

            for (zone_id, geo_list) in gz_map.iter() {
                for gz in geo_list {
                    if gz.country_code == target_country {
                        if let (Some(target_prov), Some(gz_prov)) =
                            (&ctx.province_code, &gz.province_code)
                        {
                            if target_prov.to_lowercase() == gz_prov.to_lowercase() {
                                eligible_zone_ids.push(zone_id.clone());
                                break;
                            }
                        } else {
                            eligible_zone_ids.push(zone_id.clone());
                            break;
                        }
                    }
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
        let so = {
            let map = self.shipping_options.read().unwrap();
            map.get(shipping_option_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "shipping_option",
                    id: shipping_option_id.to_string(),
                })?
        };

        if so.price_type == "flat" {
            // Flat rate default from data or zero
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
            let addr = self.create_address_internal(addr_input);
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

        // Add items
        let mut items = Vec::new();
        {
            let mut map = self.fulfillment_items.write().unwrap();
            let list = map.entry(ful_id_str.clone()).or_default();

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
                };
                items.push(Self::to_item_dto(&fi));
                list.push(fi);
            }
        }

        // Add labels
        let mut labels = Vec::new();
        {
            let mut map = self.fulfillment_labels.write().unwrap();
            let list = map.entry(ful_id_str.clone()).or_default();

            // From provider response or input
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
                labels.push(Self::to_label_dto(&fl));
                list.push(fl);
            }
        }

        let dto = Self::to_fulfillment_dto(&ful, None, items, labels);

        {
            let mut map = self.fulfillments.write().unwrap();
            map.insert(ful_id_str, ful);
        }

        Ok(dto)
    }

    pub async fn mark_as_shipped(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = {
            let map = self.fulfillments.read().unwrap();
            map.get(id).cloned().ok_or_else(|| Error::NotFound {
                entity: "fulfillment",
                id: id.to_string(),
            })?
        };

        ful.shipped_at = Some(Utc::now());
        ful.updated_at = Utc::now();

        {
            let mut map = self.fulfillments.write().unwrap();
            map.insert(id.to_string(), ful);
        }

        self.get_fulfillment(id).await
    }

    pub async fn mark_as_delivered(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = {
            let map = self.fulfillments.read().unwrap();
            map.get(id).cloned().ok_or_else(|| Error::NotFound {
                entity: "fulfillment",
                id: id.to_string(),
            })?
        };

        ful.delivered_at = Some(Utc::now());
        ful.updated_at = Utc::now();

        {
            let mut map = self.fulfillments.write().unwrap();
            map.insert(id.to_string(), ful);
        }

        self.get_fulfillment(id).await
    }

    pub async fn cancel_fulfillment(&self, id: &str) -> Result<FulfillmentDTO> {
        let mut ful = {
            let map = self.fulfillments.read().unwrap();
            map.get(id).cloned().ok_or_else(|| Error::NotFound {
                entity: "fulfillment",
                id: id.to_string(),
            })?
        };

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

        {
            let mut map = self.fulfillments.write().unwrap();
            map.insert(id.to_string(), ful);
        }

        self.get_fulfillment(id).await
    }

    pub async fn get_fulfillment(&self, id: &str) -> Result<FulfillmentDTO> {
        let ful = {
            let map = self.fulfillments.read().unwrap();
            map.get(id).cloned().ok_or_else(|| Error::NotFound {
                entity: "fulfillment",
                id: id.to_string(),
            })?
        };

        let items = {
            let map = self.fulfillment_items.read().unwrap();
            map.get(id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|i| Self::to_item_dto(&i))
                .collect()
        };

        let labels = {
            let map = self.fulfillment_labels.read().unwrap();
            map.get(id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|l| Self::to_label_dto(&l))
                .collect()
        };

        let address = if let Some(addr_id) = &ful.delivery_address_id {
            let map = self.addresses.read().unwrap();
            map.get(addr_id).map(Self::to_address_dto)
        } else {
            None
        };

        Ok(Self::to_fulfillment_dto(&ful, address, items, labels))
    }

    // ── Internal Helpers ────────────────────────────────────────────────────

    fn create_address_internal(&self, input: CreateAddressInput) -> Address {
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

        let mut map = self.addresses.write().unwrap();
        map.insert(addr.id.to_string(), addr.clone());
        addr
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
