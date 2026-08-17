//! Service layer and evaluation engine for `stheno-promotion` (P4-08 through P4-15, P13-08, P13-09).

use chrono::{DateTime, Duration, Utc};
use diesel::QueryableByName;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use stheno_db::{CrudRepository, DbPool, FilterOp, FilterSpec, PgCrudRepository};
use stheno_types::{Error, Id, Result};

use crate::dto::*;
use crate::models::{
    ApplicationMethod, Campaign, CampaignBudget, CampaignBudgetUsage, Promotion, PromotionRule,
    PromotionRuleValue,
};

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    value: String,
}

#[derive(Clone)]
pub struct PromotionService {
    pool: DbPool,
    promotions: PgCrudRepository<Promotion>,
    app_methods: PgCrudRepository<ApplicationMethod>,
    rules: PgCrudRepository<PromotionRule>,
    rule_values: PgCrudRepository<PromotionRuleValue>,
    campaigns: PgCrudRepository<Campaign>,
    budgets: PgCrudRepository<CampaignBudget>,
    budget_usages: PgCrudRepository<CampaignBudgetUsage>,
    failed_cart_attempts: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
    ip_attempts: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl PromotionService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: pool.clone(),
            promotions: PgCrudRepository::new(pool.clone()),
            app_methods: PgCrudRepository::new(pool.clone()),
            rules: PgCrudRepository::new(pool.clone()),
            rule_values: PgCrudRepository::new(pool.clone()),
            campaigns: PgCrudRepository::new(pool.clone()),
            budgets: PgCrudRepository::new(pool.clone()),
            budget_usages: PgCrudRepository::new(pool),
            failed_cart_attempts: Arc::new(RwLock::new(HashMap::new())),
            ip_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
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

        let created = self.promotions.create(None, promo).await?;
        let promo_id_str = created.id.to_string();

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
        let promo = self
            .promotions
            .retrieve(None, id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "promotion",
                    id: id.to_string(),
                },
                other => other,
            })?;

        self.hydrate_promotion_dto(&promo).await
    }

    pub async fn get_promotion_by_code(&self, code: &str) -> Result<PromotionDTO> {
        let normalized = code.trim().to_uppercase();
        let spec = FilterSpec::new()
            .with_predicate("code", FilterOp::Eq(normalized))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.promotions.list(None, &spec).await?;
        let promo = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "promotion",
            id: code.to_string(),
        })?;

        self.hydrate_promotion_dto(&promo).await
    }

    pub async fn list_promotions(&self) -> Result<Vec<PromotionDTO>> {
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1000);

        let promos = self.promotions.list(None, &spec).await?;
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
        let mut promo = self
            .promotions
            .retrieve(None, id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "promotion",
                    id: id.to_string(),
                },
                other => other,
            })?;

        if let Some(code) = input.code {
            promo.code = code.trim().to_uppercase();
        }
        if let Some(ptype) = input.promotion_type {
            promo.promotion_type = ptype;
        }
        if let Some(is_auto) = input.is_automatic {
            promo.is_automatic = is_auto;
        }
        if let Some(camp_id) = input.campaign_id {
            promo.campaign_id = Some(camp_id);
        }
        if let Some(status) = input.status {
            promo.status = status;
        }
        if let Some(metadata) = input.metadata {
            promo.metadata = Some(metadata);
        }
        promo.updated_at = Utc::now();

        let updated = self.promotions.update(None, id, promo).await?;
        self.hydrate_promotion_dto(&updated).await
    }

    pub async fn delete_promotion(&self, id: &str) -> Result<()> {
        self.promotions.soft_delete(None, id).await.map(|_| ())
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
            deleted_at: None,
        };

        let created = self.app_methods.create(None, app).await?;
        let app_id_str = created.id.to_string();

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
        let spec = FilterSpec::new()
            .with_predicate("promotion_id", FilterOp::Eq(promotion_id.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);

        let list = self.app_methods.list(None, &spec).await?;
        let app = list.into_iter().next().ok_or_else(|| Error::NotFound {
            entity: "application_method",
            id: promotion_id.to_string(),
        })?;

        let app_id_str = app.id.to_string();
        let target_rules = self
            .get_rules_for_target(&app_id_str, "target_rules")
            .await?;
        let buy_rules = self.get_rules_for_target(&app_id_str, "buy_rules").await?;

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
            deleted_at: None,
        };

        let created_rule = self.rules.create(None, rule).await?;
        let rule_id_str = created_rule.id.to_string();

        let mut val_dtos = Vec::new();
        for val in input.values {
            let v = PromotionRuleValue {
                id: Id::new(),
                promotion_rule_id: rule_id_str.clone(),
                value: val.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
            };
            self.rule_values.create(None, v).await?;
            val_dtos.push(val);
        }

        Ok(PromotionRuleDTO {
            id: created_rule.id.to_string(),
            promotion_id: created_rule.promotion_id,
            application_method_id: created_rule.application_method_id,
            rule_type: created_rule.rule_type,
            attribute: created_rule.attribute,
            operator: created_rule.operator,
            description: created_rule.description,
            values: val_dtos,
            created_at: created_rule.created_at,
            updated_at: created_rule.updated_at,
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

        let created = self.campaigns.create(None, camp).await?;
        let camp_id_str = created.id.to_string();

        if let Some(b_input) = input.budget {
            self.create_campaign_budget(&camp_id_str, b_input).await?;
        }

        self.get_campaign(&camp_id_str).await
    }

    pub async fn get_campaign(&self, id: &str) -> Result<CampaignDTO> {
        let camp = match self.campaigns.retrieve(None, id).await {
            Ok(c) => c,
            Err(_) => {
                let spec = FilterSpec::new()
                    .with_predicate("campaign_identifier", FilterOp::Eq(id.to_uppercase()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1);
                let list = self.campaigns.list(None, &spec).await?;
                list.into_iter().next().ok_or_else(|| Error::NotFound {
                    entity: "campaign",
                    id: id.to_string(),
                })?
            }
        };

        let camp_id_str = camp.id.to_string();
        let budget_spec = FilterSpec::new()
            .with_predicate("campaign_id", FilterOp::Eq(camp_id_str.clone()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1);
        let budget = self
            .budgets
            .list(None, &budget_spec)
            .await?
            .into_iter()
            .next()
            .map(|b| CampaignBudgetDTO {
                id: b.id.to_string(),
                campaign_id: b.campaign_id,
                budget_type: b.budget_type,
                limit: b.limit,
                used: b.used,
                currency_code: b.currency_code,
                created_at: b.created_at,
                updated_at: b.updated_at,
            });

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
        let spec = FilterSpec::new()
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 1000);

        let campaigns = self.campaigns.list(None, &spec).await?;
        let mut list = Vec::new();
        for camp in campaigns {
            list.push(self.get_campaign(&camp.id.to_string()).await?);
        }
        Ok(list)
    }

    pub async fn update_campaign(
        &self,
        id: &str,
        input: UpdateCampaignInput,
    ) -> Result<CampaignDTO> {
        let mut camp = self
            .campaigns
            .retrieve(None, id)
            .await
            .map_err(|e| match e {
                Error::NotFound { .. } => Error::NotFound {
                    entity: "campaign",
                    id: id.to_string(),
                },
                other => other,
            })?;

        if let Some(name) = input.name {
            camp.name = name;
        }
        if let Some(ci) = input.campaign_identifier {
            camp.campaign_identifier = ci.trim().to_uppercase();
        }
        if let Some(s) = input.starts_at {
            camp.starts_at = Some(s);
        }
        if let Some(e) = input.ends_at {
            camp.ends_at = Some(e);
        }
        if let Some(cur) = input.currency {
            camp.currency = Some(cur.to_lowercase());
        }
        if let Some(metadata) = input.metadata {
            camp.metadata = Some(metadata);
        }
        camp.updated_at = Utc::now();

        let updated = self.campaigns.update(None, id, camp).await?;
        self.get_campaign(&updated.id.to_string()).await
    }

    pub async fn delete_campaign(&self, id: &str) -> Result<()> {
        self.campaigns.soft_delete(None, id).await.map(|_| ())
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
            deleted_at: None,
        };

        let created = self.budgets.create(None, budget).await?;

        Ok(CampaignBudgetDTO {
            id: created.id.to_string(),
            campaign_id: created.campaign_id,
            budget_type: created.budget_type,
            limit: created.limit,
            used: created.used,
            currency_code: created.currency_code,
            created_at: created.created_at,
            updated_at: created.updated_at,
        })
    }

    /// Atomic campaign budget consumption (P4-13, P4-14, DATA_MODEL §2.5).
    /// Prevents over-spend/over-use under heavy concurrency using atomic SQL UPDATE.
    pub async fn consume_campaign_budget(
        &self,
        campaign_id: &str,
        amount: Decimal,
        order_id: Option<&str>,
        cart_id: Option<&str>,
    ) -> Result<CampaignBudgetUsageDTO> {
        use diesel_async::RunQueryDsl;

        let amount_str = amount.to_string();
        let cid = campaign_id.to_string();
        let sql = "UPDATE promotion.campaign_budget AS t \
                   SET used = used + $1::numeric, updated_at = now() \
                   WHERE campaign_id = $2 \
                     AND (\"limit\" IS NULL OR used + $1::numeric <= \"limit\") \
                     AND deleted_at IS NULL \
                   RETURNING row_to_json(t)::text AS value";

        let result: std::result::Result<JsonRow, diesel::result::Error> = {
            let mut conn = self
                .pool
                .get()
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

            diesel::sql_query(sql)
                .bind::<diesel::sql_types::Text, _>(&amount_str)
                .bind::<diesel::sql_types::Text, _>(&cid)
                .get_result(&mut conn)
                .await
        };

        let row = match result {
            Ok(r) => r,
            Err(diesel::result::Error::NotFound) => {
                // Determine if budget is exhausted or does not exist
                let spec = FilterSpec::new()
                    .with_predicate("campaign_id", FilterOp::Eq(cid.clone()))
                    .with_predicate("deleted_at", FilterOp::IsNull)
                    .with_pagination(0, 1);
                let list = self.budgets.list(None, &spec).await?;
                if let Some(b) = list.into_iter().next() {
                    return Err(Error::Conflict {
                        code: "budget_exhausted",
                        message: format!(
                            "Campaign budget exhausted: used {} + requested {} > limit {:?}",
                            b.used, amount, b.limit
                        ),
                    });
                } else {
                    return Err(Error::NotFound {
                        entity: "campaign_budget",
                        id: cid,
                    });
                }
            }
            Err(other) => return Err(Error::Database(other.to_string())),
        };

        let budget: CampaignBudget = serde_json::from_str(&row.value)
            .map_err(|e| Error::Database(format!("Failed to deserialise budget: {e}")))?;

        let usage = CampaignBudgetUsage {
            id: Id::new(),
            campaign_budget_id: budget.id.to_string(),
            order_id: order_id.map(|s| s.to_string()),
            cart_id: cart_id.map(|s| s.to_string()),
            amount,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let created_usage = self.budget_usages.create(None, usage).await?;

        Ok(CampaignBudgetUsageDTO {
            id: created_usage.id.to_string(),
            campaign_budget_id: created_usage.campaign_budget_id,
            order_id: created_usage.order_id,
            cart_id: created_usage.cart_id,
            amount: created_usage.amount,
            created_at: created_usage.created_at,
            updated_at: created_usage.updated_at,
        })
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
                    rule.values
                        .as_slice()
                        .first()
                        .and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr > first_target
                } else {
                    false
                }
            }
            "gte" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values
                        .as_slice()
                        .first()
                        .and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr >= first_target
                } else {
                    false
                }
            }
            "lt" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values
                        .as_slice()
                        .first()
                        .and_then(|v| v.parse::<Decimal>().ok()),
                ) {
                    curr < first_target
                } else {
                    false
                }
            }
            "lte" => {
                if let (Ok(curr), Some(first_target)) = (
                    attr_val.parse::<Decimal>(),
                    rule.values
                        .as_slice()
                        .first()
                        .and_then(|v| v.parse::<Decimal>().ok()),
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

    async fn get_rules_for_target(
        &self,
        target_id: &str,
        rule_type: &str,
    ) -> Result<Vec<PromotionRuleDTO>> {
        let promo_spec = FilterSpec::new()
            .with_predicate("promotion_id", FilterOp::Eq(target_id.to_string()))
            .with_predicate("rule_type", FilterOp::Eq(rule_type.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);

        let app_spec = FilterSpec::new()
            .with_predicate("application_method_id", FilterOp::Eq(target_id.to_string()))
            .with_predicate("rule_type", FilterOp::Eq(rule_type.to_string()))
            .with_predicate("deleted_at", FilterOp::IsNull)
            .with_pagination(0, 100);

        let mut matched_rules = self.rules.list(None, &promo_spec).await?;
        let app_matched = self.rules.list(None, &app_spec).await?;
        matched_rules.extend(app_matched);

        let mut dtos = Vec::new();
        for r in matched_rules {
            let rule_id_str = r.id.to_string();
            let val_spec = FilterSpec::new()
                .with_predicate("promotion_rule_id", FilterOp::Eq(rule_id_str.clone()))
                .with_predicate("deleted_at", FilterOp::IsNull)
                .with_pagination(0, 100);

            let values = self
                .rule_values
                .list(None, &val_spec)
                .await?
                .into_iter()
                .map(|v| v.value)
                .collect();

            dtos.push(PromotionRuleDTO {
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
            });
        }

        Ok(dtos)
    }

    async fn hydrate_promotion_dto(&self, promo: &Promotion) -> Result<PromotionDTO> {
        let promo_id_str = promo.id.to_string();
        let application_method = self.get_application_method(&promo_id_str).await.ok();
        let rules = self
            .get_rules_for_target(&promo_id_str, "rules")
            .await
            .unwrap_or_default();

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
