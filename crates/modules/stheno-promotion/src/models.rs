//! Promotion entity models (DATA_MODEL §2.5).
//! 7 entities: `promotion`, `application_method`, `promotion_rule`, `promotion_rule_value`,
//! `campaign`, `campaign_budget`, `campaign_budget_usage`.
//! Strictly `pub(crate)` and serialize-only (AGENTS.md §2.9).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use stheno_types::Id;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Promotion {
    pub id: Id<Promotion>,
    pub code: String,
    pub promotion_type: String, // "standard" or "buyget"
    pub is_automatic: bool,
    pub campaign_id: Option<String>,
    pub status: String, // "draft", "active", "inactive"
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApplicationMethod {
    pub id: Id<ApplicationMethod>,
    pub promotion_id: String,
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
}

#[derive(Debug, Clone, Serialize)]
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
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PromotionRuleValue {
    pub id: Id<PromotionRuleValue>,
    pub promotion_rule_id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CampaignBudget {
    pub id: Id<CampaignBudget>,
    pub campaign_id: String,
    pub budget_type: String, // "spend" or "usage"
    pub limit: Option<Decimal>,
    pub used: Decimal,
    pub currency_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CampaignBudgetUsage {
    pub id: Id<CampaignBudgetUsage>,
    pub campaign_budget_id: String,
    pub order_id: Option<String>,
    pub cart_id: Option<String>,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
