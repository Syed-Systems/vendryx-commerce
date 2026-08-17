//! Promotion entity models (DATA_MODEL §2.5).
//! 7 entities: `promotion`, `application_method`, `promotion_rule`, `promotion_rule_value`,
//! `campaign`, `campaign_budget`, `campaign_budget_usage`.
//! Strictly `pub(crate)` (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stheno_db::CrudEntity;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Promotion {
    pub id: Id<Promotion>,
    pub code: String,
    #[serde(rename = "type")]
    pub promotion_type: String, // "standard" or "buyget"
    pub is_automatic: bool,
    pub campaign_id: Option<String>,
    pub status: String, // "draft", "active", "inactive"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ApplicationMethod {
    pub id: Id<ApplicationMethod>,
    pub promotion_id: String,
    #[serde(rename = "type")]
    pub method_type: String, // "fixed" or "percentage"
    pub target_type: String, // "items", "shipping_methods", "order"
    pub allocation: String,  // "each" or "across"
    pub value: Decimal,
    pub currency_code: Option<String>,
    pub max_quantity: Option<i32>,
    pub apply_to_quantity: Option<i32>,
    pub buy_rules_min_quantity: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PromotionRule {
    pub id: Id<PromotionRule>,
    pub promotion_id: Option<String>,
    pub application_method_id: Option<String>,
    pub rule_type: String, // "rules", "buy_rules", "target_rules"
    pub attribute: String,
    pub operator: String, // "eq", "ne", "in", "nin", "gt", "gte", "lt", "lte"
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PromotionRuleValue {
    pub id: Id<PromotionRuleValue>,
    pub promotion_rule_id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Campaign {
    pub id: Id<Campaign>,
    pub name: String,
    pub campaign_identifier: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub currency: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CampaignBudget {
    pub id: Id<CampaignBudget>,
    pub campaign_id: String,
    #[serde(rename = "type")]
    pub budget_type: String, // "spend" or "usage"
    pub limit: Option<Decimal>,
    pub used: Decimal,
    pub currency_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CampaignBudgetUsage {
    pub id: Id<CampaignBudgetUsage>,
    pub campaign_budget_id: String,
    pub order_id: Option<String>,
    pub cart_id: Option<String>,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Medusa id prefixes for this module's entities (ADR-010).
stheno_types::prefixed_id! {
    ApplicationMethod => "app_mth",
    Campaign => "camp",
    CampaignBudget => "camp_bud",
    CampaignBudgetUsage => "cbu",
    Promotion => "promo",
    PromotionRule => "pro_rule",
    PromotionRuleValue => "prval",
}

macro_rules! impl_crud_entity {
    ($t:ident, $table:literal, $entity:literal) => {
        impl CrudEntity for $t {
            const TABLE: &'static str = $table;
            const ENTITY: &'static str = $entity;

            fn id(&self) -> String {
                self.id.to_string()
            }
            fn is_deleted(&self) -> bool {
                self.deleted_at.is_some()
            }
            fn set_deleted(&mut self, deleted: bool) {
                self.deleted_at = if deleted { Some(Utc::now()) } else { None };
            }
        }
    };
}

impl_crud_entity!(Promotion, "promotion.promotion", "promotion");
impl_crud_entity!(
    ApplicationMethod,
    "promotion.application_method",
    "application_method"
);
impl_crud_entity!(PromotionRule, "promotion.promotion_rule", "promotion_rule");
impl_crud_entity!(
    PromotionRuleValue,
    "promotion.promotion_rule_value",
    "promotion_rule_value"
);
impl_crud_entity!(Campaign, "promotion.campaign", "campaign");
impl_crud_entity!(
    CampaignBudget,
    "promotion.campaign_budget",
    "campaign_budget"
);
impl_crud_entity!(
    CampaignBudgetUsage,
    "promotion.campaign_budget_usage",
    "campaign_budget_usage"
);
