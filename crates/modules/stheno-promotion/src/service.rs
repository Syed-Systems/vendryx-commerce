//! Service layer and evaluation engine for `stheno-promotion` (P4-08 through P4-15).

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    ApplicationMethod, Campaign, CampaignBudget, CampaignBudgetUsage, Promotion, PromotionRule,
    PromotionRuleValue,
};

#[derive(Clone, Default)]
pub struct PromotionService {
    promotions: Arc<RwLock<HashMap<String, Promotion>>>,
    app_methods: Arc<RwLock<HashMap<String, ApplicationMethod>>>,
    rules: Arc<RwLock<HashMap<String, Vec<PromotionRule>>>>,
    rule_values: Arc<RwLock<HashMap<String, Vec<PromotionRuleValue>>>>,
    campaigns: Arc<RwLock<HashMap<String, Campaign>>>,
    budgets: Arc<Mutex<HashMap<String, CampaignBudget>>>, // Mutex for atomic budget consumption (P4-13, P4-14)
    budget_usages: Arc<RwLock<HashMap<String, Vec<CampaignBudgetUsage>>>>,
    failed_cart_attempts: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<Utc>>>>>,
    ip_attempts: Arc<RwLock<HashMap<String, Vec<chrono::DateTime<Utc>>>>>,
}

impl PromotionService {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Promotion CRUD ───────────────────────────────────────────────────────

    pub async fn create_promotion(&self, input: CreatePromotionInput) -> Result<PromotionDTO> {
        let id: Id<Promotion> = Id::new();
        let now = Utc::now();
        let promo = Promotion {
            id,
            code: input.code.trim().to_uppercase(),
            promotion_type: input
                .promotion_type
                .unwrap_or_else(|| "standard".to_string()),
            is_automatic: input.is_automatic.unwrap_or(false),
            campaign_id: input.campaign_id,
            status: input.status.unwrap_or_else(|| "active".to_string()),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let promo_id_str = promo.id.to_string();
        {
            let mut map = self.promotions.write().unwrap();
            map.insert(promo_id_str.clone(), promo);
        }

        if let Some(app_input) = input.application_method {
            self.create_application_method(&promo_id_str, app_input)
                .await?;
        }

        if let Some(rules) = input.rules {
            for rule_input in rules {
                self.create_promotion_rule(Some(&promo_id_str), None, rule_input)
                    .await?;
            }
        }

        self.get_promotion(&promo_id_str).await
    }

    pub async fn get_promotion(&self, id: &str) -> Result<PromotionDTO> {
        let promo = {
            let map = self.promotions.read().unwrap();
            map.values()
                .find(|p| {
                    (p.id.to_string() == id || p.id.to_string().ends_with(id))
                        && p.deleted_at.is_none()
                })
                .cloned()
                .or_else(|| map.get(id).filter(|p| p.deleted_at.is_none()).cloned())
                .ok_or_else(|| Error::NotFound {
                    entity: "promotion",
                    id: id.to_string(),
                })?
        };

        self.hydrate_promotion_dto(&promo).await
    }

    pub async fn get_promotion_by_code(&self, code: &str) -> Result<PromotionDTO> {
        let normalized = code.trim().to_uppercase();
        let promo = {
            let map = self.promotions.read().unwrap();
            map.values()
                .find(|p| constant_time_eq_str(&p.code, &normalized) && p.deleted_at.is_none())
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "promotion",
                    id: code.to_string(),
                })?
        };

        self.hydrate_promotion_dto(&promo).await
    }

    pub async fn list_promotions(&self) -> Result<Vec<PromotionDTO>> {
        let promos: Vec<Promotion> = {
            let map = self.promotions.read().unwrap();
            map.values().filter(|p| p.deleted_at.is_none()).cloned().collect()
        };

        let mut list = Vec::new();
        for promo in promos {
            list.push(self.hydrate_promotion_dto(&promo).await?);
        }
        Ok(list)
    }

    pub async fn update_promotion(
        &self,
        id: &str,
        input: UpdatePromotionInput,
    ) -> Result<PromotionDTO> {
        let promo_id_str = {
            let mut map = self.promotions.write().unwrap();
            let p = map
                .values_mut()
                .find(|p| {
                    (p.id.to_string() == id || p.id.to_string().ends_with(id))
                        && p.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "promotion",
                    id: id.to_string(),
                })?;

            if let Some(code) = input.code {
                p.code = code.trim().to_uppercase();
            }
            if let Some(ptype) = input.promotion_type {
                p.promotion_type = ptype;
            }
            if let Some(is_auto) = input.is_automatic {
                p.is_automatic = is_auto;
            }
            if let Some(camp_id) = input.campaign_id {
                p.campaign_id = Some(camp_id);
            }
            if let Some(status) = input.status {
                p.status = status;
            }
            if let Some(metadata) = input.metadata {
                p.metadata = Some(metadata);
            }
            p.updated_at = Utc::now();
            p.id.to_string()
        };

        self.get_promotion(&promo_id_str).await
    }

    pub async fn delete_promotion(&self, id: &str) -> Result<()> {
        let mut map = self.promotions.write().unwrap();
        let p = map
            .values_mut()
            .find(|p| {
                (p.id.to_string() == id || p.id.to_string().ends_with(id)) && p.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "promotion",
                id: id.to_string(),
            })?;
        p.deleted_at = Some(Utc::now());
        Ok(())
    }

    // ── Application Method CRUD ──────────────────────────────────────────────

    pub async fn create_application_method(
        &self,
        promotion_id: &str,
        input: CreateApplicationMethodInput,
    ) -> Result<ApplicationMethodDTO> {
        let id: Id<ApplicationMethod> = Id::new();
        let app = ApplicationMethod {
            id,
            promotion_id: promotion_id.to_string(),
            method_type: input.method_type,
            target_type: input.target_type,
            allocation: input.allocation.unwrap_or_else(|| "across".to_string()),
            value: input.value,
            currency_code: input.currency_code.map(|c| c.to_lowercase()),
            max_quantity: input.max_quantity,
            apply_to_quantity: input.apply_to_quantity,
            buy_rules_min_quantity: input.buy_rules_min_quantity,
            metadata: input.metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let app_id_str = app.id.to_string();
        {
            let mut map = self.app_methods.write().unwrap();
            map.insert(promotion_id.to_string(), app);
        }

        if let Some(target_rules) = input.target_rules {
            for mut r in target_rules {
                r.rule_type = Some("target_rules".to_string());
                self.create_promotion_rule(None, Some(&app_id_str), r)
                    .await?;
            }
        }

        if let Some(buy_rules) = input.buy_rules {
            for mut r in buy_rules {
                r.rule_type = Some("buy_rules".to_string());
                self.create_promotion_rule(None, Some(&app_id_str), r)
                    .await?;
            }
        }

        self.get_application_method(promotion_id).await
    }

    pub async fn get_application_method(&self, promotion_id: &str) -> Result<ApplicationMethodDTO> {
        let app = {
            let map = self.app_methods.read().unwrap();
            map.get(promotion_id)
                .cloned()
                .ok_or_else(|| Error::NotFound {
                    entity: "application_method",
                    id: promotion_id.to_string(),
                })?
        };

        let app_id_str = app.id.to_string();
        let target_rules = self.get_rules_for_target(&app_id_str, "target_rules");
        let buy_rules = self.get_rules_for_target(&app_id_str, "buy_rules");

        Ok(ApplicationMethodDTO {
            id: app.id.to_string(),
            promotion_id: app.promotion_id,
            method_type: app.method_type,
            target_type: app.target_type,
            allocation: app.allocation,
            value: app.value,
            currency_code: app.currency_code,
            max_quantity: app.max_quantity,
            apply_to_quantity: app.apply_to_quantity,
            buy_rules_min_quantity: app.buy_rules_min_quantity,
            target_rules,
            buy_rules,
            metadata: app.metadata,
            created_at: app.created_at,
            updated_at: app.updated_at,
        })
    }

    // ── Rules CRUD ───────────────────────────────────────────────────────────

    pub async fn create_promotion_rule(
        &self,
        promotion_id: Option<&str>,
        application_method_id: Option<&str>,
        input: CreatePromotionRuleInput,
    ) -> Result<PromotionRuleDTO> {
        let id: Id<PromotionRule> = Id::new();
        let rule = PromotionRule {
            id,
            promotion_id: promotion_id.map(|s| s.to_string()),
            application_method_id: application_method_id.map(|s| s.to_string()),
            rule_type: input.rule_type.unwrap_or_else(|| "rules".to_string()),
            attribute: input.attribute,
            operator: input.operator.to_lowercase(),
            description: input.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let rule_id_str = rule.id.to_string();
        let target_key = promotion_id
            .or(application_method_id)
            .unwrap_or_default()
            .to_string();

        {
            let mut map = self.rules.write().unwrap();
            map.entry(target_key).or_default().push(rule.clone());
        }

        let mut val_dtos = Vec::new();
        {
            let mut val_map = self.rule_values.write().unwrap();
            let list = val_map.entry(rule_id_str.clone()).or_default();
            for val in input.values {
                let v = PromotionRuleValue {
                    id: Id::new(),
                    promotion_rule_id: rule_id_str.clone(),
                    value: val.clone(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                list.push(v);
                val_dtos.push(val);
            }
        }

        Ok(PromotionRuleDTO {
            id: rule.id.to_string(),
            promotion_id: rule.promotion_id,
            application_method_id: rule.application_method_id,
            rule_type: rule.rule_type,
            attribute: rule.attribute,
            operator: rule.operator,
            description: rule.description,
            values: val_dtos,
            created_at: rule.created_at,
            updated_at: rule.updated_at,
        })
    }

    // ── Campaign & Budget CRUD ───────────────────────────────────────────────

    pub async fn create_campaign(&self, input: CreateCampaignInput) -> Result<CampaignDTO> {
        let id: Id<Campaign> = Id::new();
        let now = Utc::now();
        let camp = Campaign {
            id,
            name: input.name,
            campaign_identifier: input.campaign_identifier.trim().to_uppercase(),
            starts_at: input.starts_at,
            ends_at: input.ends_at,
            currency: input.currency.map(|c| c.to_lowercase()),
            metadata: input.metadata,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let camp_id_str = camp.id.to_string();
        {
            let mut map = self.campaigns.write().unwrap();
            map.insert(camp_id_str.clone(), camp);
        }

        if let Some(b_input) = input.budget {
            self.create_campaign_budget(&camp_id_str, b_input).await?;
        }

        self.get_campaign(&camp_id_str).await
    }

    pub async fn get_campaign(&self, id: &str) -> Result<CampaignDTO> {
        let camp = {
            let map = self.campaigns.read().unwrap();
            map.values()
                .find(|c| {
                    (c.id.to_string() == id || c.id.to_string().ends_with(id))
                        && c.deleted_at.is_none()
                })
                .cloned()
                .or_else(|| map.get(id).filter(|c| c.deleted_at.is_none()).cloned())
                .ok_or_else(|| Error::NotFound {
                    entity: "campaign",
                    id: id.to_string(),
                })?
        };

        let camp_id_str = camp.id.to_string();
        let budget = {
            let map = self.budgets.lock().unwrap();
            map.get(&camp_id_str).map(|b| CampaignBudgetDTO {
                id: b.id.to_string(),
                campaign_id: b.campaign_id.clone(),
                budget_type: b.budget_type.clone(),
                limit: b.limit,
                used: b.used,
                currency_code: b.currency_code.clone(),
                created_at: b.created_at,
                updated_at: b.updated_at,
            })
        };

        Ok(CampaignDTO {
            id: camp.id.to_string(),
            name: camp.name,
            campaign_identifier: camp.campaign_identifier,
            starts_at: camp.starts_at,
            ends_at: camp.ends_at,
            currency: camp.currency,
            budget,
            metadata: camp.metadata,
            created_at: camp.created_at,
            updated_at: camp.updated_at,
        })
    }

    pub async fn list_campaigns(&self) -> Result<Vec<CampaignDTO>> {
        let ids: Vec<String> = {
            let map = self.campaigns.read().unwrap();
            map.values()
                .filter(|c| c.deleted_at.is_none())
                .map(|c| c.id.to_string())
                .collect()
        };

        let mut list = Vec::new();
        for id in ids {
            list.push(self.get_campaign(&id).await?);
        }
        Ok(list)
    }

    pub async fn update_campaign(
        &self,
        id: &str,
        input: UpdateCampaignInput,
    ) -> Result<CampaignDTO> {
        let camp_id_str = {
            let mut map = self.campaigns.write().unwrap();
            let c = map
                .values_mut()
                .find(|c| {
                    (c.id.to_string() == id || c.id.to_string().ends_with(id))
                        && c.deleted_at.is_none()
                })
                .ok_or_else(|| Error::NotFound {
                    entity: "campaign",
                    id: id.to_string(),
                })?;

            if let Some(name) = input.name {
                c.name = name;
            }
            if let Some(ci) = input.campaign_identifier {
                c.campaign_identifier = ci.trim().to_uppercase();
            }
            if let Some(s) = input.starts_at {
                c.starts_at = Some(s);
            }
            if let Some(e) = input.ends_at {
                c.ends_at = Some(e);
            }
            if let Some(cur) = input.currency {
                c.currency = Some(cur.to_lowercase());
            }
            if let Some(metadata) = input.metadata {
                c.metadata = Some(metadata);
            }
            c.updated_at = Utc::now();
            c.id.to_string()
        };

        self.get_campaign(&camp_id_str).await
    }

    pub async fn delete_campaign(&self, id: &str) -> Result<()> {
        let mut map = self.campaigns.write().unwrap();
        let c = map
            .values_mut()
            .find(|c| {
                (c.id.to_string() == id || c.id.to_string().ends_with(id))
                    && c.deleted_at.is_none()
            })
            .ok_or_else(|| Error::NotFound {
                entity: "campaign",
                id: id.to_string(),
            })?;
        c.deleted_at = Some(Utc::now());
        Ok(())
    }

    pub async fn create_campaign_budget(
        &self,
        campaign_id: &str,
        input: CreateCampaignBudgetInput,
    ) -> Result<CampaignBudgetDTO> {
        let id: Id<CampaignBudget> = Id::new();
        let budget = CampaignBudget {
            id,
            campaign_id: campaign_id.to_string(),
            budget_type: input.budget_type.to_lowercase(),
            limit: input.limit,
            used: dec!(0),
            currency_code: input.currency_code.map(|c| c.to_lowercase()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = CampaignBudgetDTO {
            id: budget.id.to_string(),
            campaign_id: budget.campaign_id.clone(),
            budget_type: budget.budget_type.clone(),
            limit: budget.limit,
            used: budget.used,
            currency_code: budget.currency_code.clone(),
            created_at: budget.created_at,
            updated_at: budget.updated_at,
        };

        let mut map = self.budgets.lock().unwrap();
        map.insert(campaign_id.to_string(), budget);
        Ok(dto)
    }

    /// Atomic campaign budget consumption (P4-13, P4-14, DATA_MODEL §2.5).
    /// Prevents over-spend/over-use under heavy concurrency.
    pub async fn consume_campaign_budget(
        &self,
        campaign_id: &str,
        amount: Decimal,
        order_id: Option<&str>,
        cart_id: Option<&str>,
    ) -> Result<CampaignBudgetUsageDTO> {
        let mut map = self.budgets.lock().unwrap();
        let budget = map.get_mut(campaign_id).ok_or_else(|| Error::NotFound {
            entity: "campaign_budget",
            id: campaign_id.to_string(),
        })?;

        // Atomic check and increment
        if let Some(limit) = budget.limit {
            if budget.used + amount > limit {
                return Err(Error::Conflict {
                    code: "budget_exhausted",
                    message: format!(
                        "Campaign budget exhausted: used {} + requested {} > limit {}",
                        budget.used, amount, limit
                    ),
                });
            }
        }

        budget.used += amount;
        budget.updated_at = Utc::now();
        let budget_id = budget.id.to_string();

        let usage = CampaignBudgetUsage {
            id: Id::new(),
            campaign_budget_id: budget_id.clone(),
            order_id: order_id.map(|s| s.to_string()),
            cart_id: cart_id.map(|s| s.to_string()),
            amount,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let dto = CampaignBudgetUsageDTO {
            id: usage.id.to_string(),
            campaign_budget_id: usage.campaign_budget_id.clone(),
            order_id: usage.order_id.clone(),
            cart_id: usage.cart_id.clone(),
            amount: usage.amount,
            created_at: usage.created_at,
            updated_at: usage.updated_at,
        };

        let mut usage_map = self.budget_usages.write().unwrap();
        usage_map.entry(budget_id).or_default().push(usage);

        Ok(dto)
    }

    // ── Rule Engine (P4-09: 8 Operators) ─────────────────────────────────────

    pub fn evaluate_rule(
        &self,
        rule: &PromotionRuleDTO,
        context: &PromotionEvaluationContext,
    ) -> bool {
        let attr_val = match context.attributes.get(&rule.attribute) {
            Some(v) => v.as_str(),
            None => "",
        };

        match rule.operator.as_str() {
            "eq" => rule.values.iter().any(|v| v.eq_ignore_ascii_case(attr_val)),
            "ne" => !rule.values.iter().any(|v| v.eq_ignore_ascii_case(attr_val)),
            "in" => rule.values.iter().any(|v| v.eq_ignore_ascii_case(attr_val)),
            "nin" => !rule.values.iter().any(|v| v.eq_ignore_ascii_case(attr_val)),
            "gt" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values.first().and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr > first_target
                } else {
                    false
                }
            }
            "gte" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values.first().and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr >= first_target
                } else {
                    false
                }
            }
            "lt" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values.first().and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr < first_target
                } else {
                    false
                }
            }
            "lte" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values.first().and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr <= first_target
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn matches_item_rule(&self, rule: &PromotionRuleDTO, item: &PromotionItemContext) -> bool {
        let item_val = match rule.attribute.as_str() {
            "product_id" => item.product_id.as_deref().unwrap_or(""),
            "product_category_id" => item.product_category_id.as_deref().unwrap_or(""),
            "product_collection_id" => item.product_collection_id.as_deref().unwrap_or(""),
            "product_type_id" => item.product_type_id.as_deref().unwrap_or(""),
            _ => "",
        };

        match rule.operator.as_str() {
            "eq" | "in" => rule.values.iter().any(|v| v.eq_ignore_ascii_case(item_val)),
            "ne" | "nin" => !rule.values.iter().any(|v| v.eq_ignore_ascii_case(item_val)),
            _ => false,
        }
    }

    // ── Application Evaluation & Stacking (P4-10, P4-11, P4-12) ───────────────

    pub async fn evaluate_promotions(
        &self,
        promotions: &[PromotionDTO],
        context: &PromotionEvaluationContext,
    ) -> Vec<ComputedDiscountAdjustment> {
        let mut sorted_promotions = promotions.to_vec();
        // Deterministic stacking order (P4-12):
        // 1. Fixed amount discounts before Percentage discounts
        // 2. Target: items before order before shipping_methods
        // 3. Code alphabetical tie-breaker
        sorted_promotions.sort_by(|a, b| {
            let type_rank = |p: &PromotionDTO| -> (u8, u8) {
                if let Some(app) = &p.application_method {
                    let target_rank = match app.target_type.as_str() {
                        "items" => 1,
                        "order" => 2,
                        "shipping_methods" => 3,
                        _ => 4,
                    };
                    let method_rank = match app.method_type.as_str() {
                        "fixed" => 1,
                        "percentage" => 2,
                        _ => 3,
                    };
                    (method_rank, target_rank)
                } else {
                    (9, 9)
                }
            };
            type_rank(a)
                .cmp(&type_rank(b))
                .then_with(|| a.code.cmp(&b.code))
        });

        let mut adjustments = Vec::new();
        let mut item_remaining_balances: HashMap<String, Decimal> = context
            .items
            .iter()
            .map(|i| (i.id.clone(), i.subtotal))
            .collect();
        let mut shipping_remaining_balances: HashMap<String, Decimal> = context
            .shipping_methods
            .iter()
            .map(|s| (s.id.clone(), s.amount))
            .collect();

        for promo in &sorted_promotions {
            if promo.status != "active" {
                continue;
            }

            // Check eligibility rules
            let eligible = promo.rules.iter().all(|r| self.evaluate_rule(r, context));
            if !eligible {
                continue;
            }

            let app = match &promo.application_method {
                Some(a) => a,
                None => continue,
            };

            if promo.promotion_type == "buyget" {
                // Buyget evaluation (P4-11)
                self.evaluate_buyget(
                    promo,
                    app,
                    context,
                    &mut item_remaining_balances,
                    &mut adjustments,
                );
            } else {
                // Standard application evaluation (P4-10)
                self.evaluate_standard(
                    promo,
                    app,
                    context,
                    &mut item_remaining_balances,
                    &mut shipping_remaining_balances,
                    &mut adjustments,
                );
            }
        }

        adjustments
    }

    fn evaluate_standard(
        &self,
        promo: &PromotionDTO,
        app: &ApplicationMethodDTO,
        context: &PromotionEvaluationContext,
        item_balances: &mut HashMap<String, Decimal>,
        shipping_balances: &mut HashMap<String, Decimal>,
        adjustments: &mut Vec<ComputedDiscountAdjustment>,
    ) {
        match app.target_type.as_str() {
            "items" => {
                let matching_items: Vec<&PromotionItemContext> = context
                    .items
                    .iter()
                    .filter(|item| {
                        item.is_discountable
                            && app
                                .target_rules
                                .iter()
                                .all(|r| self.matches_item_rule(r, item))
                    })
                    .collect();

                if matching_items.is_empty() {
                    return;
                }

                if app.allocation == "each" {
                    // Fixed or percentage applied to EACH eligible item
                    for item in matching_items {
                        let remaining = item_balances.get_mut(&item.id).unwrap();
                        if *remaining <= dec!(0) {
                            continue;
                        }

                        let discount_amount = match app.method_type.as_str() {
                            "fixed" => {
                                let max_qty =
                                    app.max_quantity.unwrap_or(item.quantity).min(item.quantity);
                                (app.value * Decimal::from(max_qty)).min(*remaining)
                            }
                            "percentage" => (*remaining * (app.value / dec!(100)))
                                .round_dp(2)
                                .min(*remaining),
                            _ => dec!(0),
                        };

                        if discount_amount > dec!(0) {
                            *remaining -= discount_amount;
                            adjustments.push(ComputedDiscountAdjustment {
                                line_item_id: Some(item.id.clone()),
                                shipping_method_id: None,
                                amount: discount_amount,
                                promotion_id: promo.id.clone(),
                                code: promo.code.clone(),
                                description: Some(format!("Discount from {}", promo.code)),
                            });
                        }
                    }
                } else {
                    // Allocation: across (applies total value distributed proportionally)
                    let total_matching_subtotal: Decimal = matching_items
                        .iter()
                        .map(|i| *item_balances.get(&i.id).unwrap_or(&dec!(0)))
                        .sum();

                    if total_matching_subtotal <= dec!(0) {
                        return;
                    }

                    let total_discount_to_distribute = match app.method_type.as_str() {
                        "fixed" => app.value.min(total_matching_subtotal),
                        "percentage" => (total_matching_subtotal * (app.value / dec!(100)))
                            .round_dp(2)
                            .min(total_matching_subtotal),
                        _ => dec!(0),
                    };

                    let mut remaining_discount = total_discount_to_distribute;
                    for (idx, item) in matching_items.iter().enumerate() {
                        let remaining = item_balances.get_mut(&item.id).unwrap();
                        let share = if idx == matching_items.len() - 1 {
                            remaining_discount.min(*remaining)
                        } else {
                            let prop = *remaining / total_matching_subtotal;
                            (total_discount_to_distribute * prop)
                                .round_dp(2)
                                .min(*remaining)
                                .min(remaining_discount)
                        };

                        if share > dec!(0) {
                            *remaining -= share;
                            remaining_discount -= share;
                            adjustments.push(ComputedDiscountAdjustment {
                                line_item_id: Some(item.id.clone()),
                                shipping_method_id: None,
                                amount: share,
                                promotion_id: promo.id.clone(),
                                code: promo.code.clone(),
                                description: Some(format!("Discount from {}", promo.code)),
                            });
                        }
                    }
                }
            }
            "shipping_methods" => {
                for sm in &context.shipping_methods {
                    let remaining = match shipping_balances.get_mut(&sm.id) {
                        Some(b) => b,
                        None => continue,
                    };
                    if *remaining <= dec!(0) {
                        continue;
                    }

                    let discount_amount = match app.method_type.as_str() {
                        "fixed" => app.value.min(*remaining),
                        "percentage" => (*remaining * (app.value / dec!(100)))
                            .round_dp(2)
                            .min(*remaining),
                        _ => dec!(0),
                    };

                    if discount_amount > dec!(0) {
                        *remaining -= discount_amount;
                        adjustments.push(ComputedDiscountAdjustment {
                            line_item_id: None,
                            shipping_method_id: Some(sm.id.clone()),
                            amount: discount_amount,
                            promotion_id: promo.id.clone(),
                            code: promo.code.clone(),
                            description: Some(format!("Shipping discount from {}", promo.code)),
                        });
                    }
                }
            }
            "order" => {
                // Distributed across all items
                let total_item_subtotal: Decimal = item_balances.values().copied().sum();
                if total_item_subtotal <= dec!(0) {
                    return;
                }

                let total_discount = match app.method_type.as_str() {
                    "fixed" => app.value.min(total_item_subtotal),
                    "percentage" => (total_item_subtotal * (app.value / dec!(100)))
                        .round_dp(2)
                        .min(total_item_subtotal),
                    _ => dec!(0),
                };

                let mut remaining_discount = total_discount;
                let item_ids: Vec<String> = context.items.iter().map(|i| i.id.clone()).collect();
                for (idx, item_id) in item_ids.iter().enumerate() {
                    let remaining = item_balances.get_mut(item_id).unwrap();
                    let share = if idx == item_ids.len() - 1 {
                        remaining_discount.min(*remaining)
                    } else {
                        let prop = *remaining / total_item_subtotal;
                        (total_discount * prop)
                            .round_dp(2)
                            .min(*remaining)
                            .min(remaining_discount)
                    };

                    if share > dec!(0) {
                        *remaining -= share;
                        remaining_discount -= share;
                        adjustments.push(ComputedDiscountAdjustment {
                            line_item_id: Some(item_id.clone()),
                            shipping_method_id: None,
                            amount: share,
                            promotion_id: promo.id.clone(),
                            code: promo.code.clone(),
                            description: Some(format!("Order discount from {}", promo.code)),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn evaluate_buyget(
        &self,
        promo: &PromotionDTO,
        app: &ApplicationMethodDTO,
        context: &PromotionEvaluationContext,
        item_balances: &mut HashMap<String, Decimal>,
        adjustments: &mut Vec<ComputedDiscountAdjustment>,
    ) {
        let min_buy_qty = app.buy_rules_min_quantity.unwrap_or(1);
        let buy_items: Vec<&PromotionItemContext> = context
            .items
            .iter()
            .filter(|item| {
                app.buy_rules
                    .iter()
                    .all(|r| self.matches_item_rule(r, item))
            })
            .collect();

        let total_buy_qty: i32 = buy_items.iter().map(|i| i.quantity).sum();
        if total_buy_qty < min_buy_qty {
            return;
        }

        let apply_qty = app.apply_to_quantity.unwrap_or(1);
        let target_items: Vec<&PromotionItemContext> = context
            .items
            .iter()
            .filter(|item| {
                app.target_rules
                    .iter()
                    .all(|r| self.matches_item_rule(r, item))
            })
            .collect();

        let mut eligible_qty_remaining = apply_qty;
        for target in target_items {
            if eligible_qty_remaining <= 0 {
                break;
            }

            let qty_to_discount = target.quantity.min(eligible_qty_remaining);
            let remaining_balance = match item_balances.get_mut(&target.id) {
                Some(b) => b,
                None => continue,
            };

            let discount_amount = match app.method_type.as_str() {
                "percentage" => {
                    let target_subtotal = target.unit_price * Decimal::from(qty_to_discount);
                    (target_subtotal * (app.value / dec!(100)))
                        .round_dp(2)
                        .min(*remaining_balance)
                }
                "fixed" => (app.value * Decimal::from(qty_to_discount)).min(*remaining_balance),
                _ => dec!(0),
            };

            if discount_amount > dec!(0) {
                *remaining_balance -= discount_amount;
                eligible_qty_remaining -= qty_to_discount;
                adjustments.push(ComputedDiscountAdjustment {
                    line_item_id: Some(target.id.clone()),
                    shipping_method_id: None,
                    amount: discount_amount,
                    promotion_id: promo.id.clone(),
                    code: promo.code.clone(),
                    description: Some(format!("BuyGet discount from {}", promo.code)),
                });
            }
        }
    }

    // ── Promo Code Abuse Limits (P4-15, F10, F31) ────────────────────────────

    pub fn record_failed_promo_attempt(&self, cart_id: &str) {
        let mut map = self.failed_cart_attempts.write().unwrap();
        map.entry(cart_id.to_string()).or_default().push(Utc::now());
    }

    pub fn check_cart_promo_abuse_limit(
        &self,
        cart_id: &str,
        max_failures_per_hour: usize,
    ) -> bool {
        let mut map = self.failed_cart_attempts.write().unwrap();
        let one_hour_ago = Utc::now() - Duration::hours(1);
        let timestamps = map.entry(cart_id.to_string()).or_default();
        timestamps.retain(|t| *t > one_hour_ago);
        timestamps.len() < max_failures_per_hour
    }

    pub fn check_ip_promo_rate_limit(&self, ip: &str, max_per_minute: usize) -> bool {
        let mut map = self.ip_attempts.write().unwrap();
        let one_minute_ago = Utc::now() - Duration::minutes(1);
        let timestamps = map.entry(ip.to_string()).or_default();
        timestamps.retain(|t| *t > one_minute_ago);

        if timestamps.len() >= max_per_minute {
            false
        } else {
            timestamps.push(Utc::now());
            true
        }
    }

    // ── Internal Helpers ─────────────────────────────────────────────────────

    fn get_rules_for_target(&self, target_id: &str, rule_type: &str) -> Vec<PromotionRuleDTO> {
        let map = self.rules.read().unwrap();
        let val_map = self.rule_values.read().unwrap();

        map.get(target_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.rule_type == rule_type)
            .map(|r| {
                let rule_id_str = r.id.to_string();
                let values: Vec<String> = val_map
                    .get(&rule_id_str)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.value)
                    .collect();

                PromotionRuleDTO {
                    id: rule_id_str,
                    promotion_id: r.promotion_id,
                    application_method_id: r.application_method_id,
                    rule_type: r.rule_type,
                    attribute: r.attribute,
                    operator: r.operator,
                    description: r.description,
                    values,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
            })
            .collect()
    }

    async fn hydrate_promotion_dto(&self, promo: &Promotion) -> Result<PromotionDTO> {
        let promo_id_str = promo.id.to_string();
        let application_method = self.get_application_method(&promo_id_str).await.ok();
        let rules = self.get_rules_for_target(&promo_id_str, "rules");

        Ok(PromotionDTO {
            id: promo.id.to_string(),
            code: promo.code.clone(),
            promotion_type: promo.promotion_type.clone(),
            is_automatic: promo.is_automatic,
            campaign_id: promo.campaign_id.clone(),
            status: promo.status.clone(),
            application_method,
            rules,
            metadata: promo.metadata.clone(),
            created_at: promo.created_at,
            updated_at: promo.updated_at,
        })
    }
}

/// Constant-time string equality comparison (F10, F31)
pub fn constant_time_eq_str(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
