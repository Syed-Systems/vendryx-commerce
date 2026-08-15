//! DTOs and evaluation context for `stheno-promotion` (DATA_MODEL §2.5).

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Promotion Rule DTOs ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRuleValueDTO {
    pub id: String,
    pub promotion_rule_id: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRuleDTO {
    pub id: String,
    pub promotion_id: Option<String>,
    pub application_method_id: Option<String>,
    pub rule_type: String, // "rules", "buy_rules", "target_rules"
    pub attribute: String,
    pub operator: String, // "eq", "ne", "in", "nin", "gt", "gte", "lt", "lte"
    pub description: Option<String>,
    pub values: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromotionRuleInput {
    pub rule_type: Option<String>, // defaults to "rules"
    pub attribute: String,
    pub operator: String,
    pub description: Option<String>,
    pub values: Vec<String>,
}

// ── Application Method DTOs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMethodDTO {
    pub id: String,
    pub promotion_id: String,
    pub method_type: String, // "fixed" or "percentage"
    pub target_type: String, // "items", "shipping_methods", "order"
    pub allocation: String,  // "each" or "across"
    pub value: Decimal,
    pub currency_code: Option<String>,
    pub max_quantity: Option<i32>,
    pub apply_to_quantity: Option<i32>,
    pub buy_rules_min_quantity: Option<i32>,
    pub target_rules: Vec<PromotionRuleDTO>,
    pub buy_rules: Vec<PromotionRuleDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApplicationMethodInput {
    pub method_type: String,
    pub target_type: String,
    pub allocation: Option<String>,
    pub value: Decimal,
    pub currency_code: Option<String>,
    pub max_quantity: Option<i32>,
    pub apply_to_quantity: Option<i32>,
    pub buy_rules_min_quantity: Option<i32>,
    pub target_rules: Option<Vec<CreatePromotionRuleInput>>,
    pub buy_rules: Option<Vec<CreatePromotionRuleInput>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateApplicationMethodInput {
    pub method_type: Option<String>,
    pub target_type: Option<String>,
    pub allocation: Option<String>,
    pub value: Option<Decimal>,
    pub currency_code: Option<String>,
    pub max_quantity: Option<i32>,
    pub apply_to_quantity: Option<i32>,
    pub buy_rules_min_quantity: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

// ── Campaign & Budget DTOs ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignBudgetUsageDTO {
    pub id: String,
    pub campaign_budget_id: String,
    pub order_id: Option<String>,
    pub cart_id: Option<String>,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignBudgetDTO {
    pub id: String,
    pub campaign_id: String,
    pub budget_type: String, // "spend" or "usage"
    pub limit: Option<Decimal>,
    pub used: Decimal,
    pub currency_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignBudgetInput {
    pub budget_type: String,
    pub limit: Option<Decimal>,
    pub currency_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignDTO {
    pub id: String,
    pub name: String,
    pub campaign_identifier: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub currency: Option<String>,
    pub budget: Option<CampaignBudgetDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaignInput {
    pub name: String,
    pub campaign_identifier: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub currency: Option<String>,
    pub budget: Option<CreateCampaignBudgetInput>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCampaignInput {
    pub name: Option<String>,
    pub campaign_identifier: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub currency: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── Promotion DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionDTO {
    pub id: String,
    pub code: String,
    pub promotion_type: String, // "standard" or "buyget"
    pub is_automatic: bool,
    pub campaign_id: Option<String>,
    pub status: String, // "draft", "active", "inactive"
    pub application_method: Option<ApplicationMethodDTO>,
    pub rules: Vec<PromotionRuleDTO>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePromotionInput {
    pub code: String,
    pub promotion_type: Option<String>,
    pub is_automatic: Option<bool>,
    pub campaign_id: Option<String>,
    pub status: Option<String>,
    pub application_method: Option<CreateApplicationMethodInput>,
    pub rules: Option<Vec<CreatePromotionRuleInput>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePromotionInput {
    pub code: Option<String>,
    pub promotion_type: Option<String>,
    pub is_automatic: Option<bool>,
    pub campaign_id: Option<String>,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ── Evaluation Context & Result ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionItemContext {
    pub id: String,
    pub product_id: Option<String>,
    pub product_category_id: Option<String>,
    pub product_collection_id: Option<String>,
    pub product_type_id: Option<String>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub is_discountable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionShippingContext {
    pub id: String,
    pub shipping_option_id: Option<String>,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromotionEvaluationContext {
    pub attributes: HashMap<String, String>, // e.g. currency_code, customer_id, etc.
    pub items: Vec<PromotionItemContext>,
    pub shipping_methods: Vec<PromotionShippingContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComputedDiscountAdjustment {
    pub line_item_id: Option<String>,
    pub shipping_method_id: Option<String>,
    pub amount: Decimal,
    pub promotion_id: String,
    pub code: String,
    pub description: Option<String>,
}
