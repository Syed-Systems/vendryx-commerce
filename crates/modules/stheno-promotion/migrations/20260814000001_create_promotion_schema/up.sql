-- Phase: expand
-- Creates the promotion module schema and its 7 owned tables.

CREATE SCHEMA IF NOT EXISTS promotion;

-- 1. campaign
CREATE TABLE IF NOT EXISTS promotion.campaign (
    id text PRIMARY KEY,
    name text NOT NULL,
    campaign_identifier text NOT NULL UNIQUE,
    starts_at timestamptz NULL,
    ends_at timestamptz NULL,
    currency text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /admin/campaigns?campaign_identifier=
CREATE INDEX IF NOT EXISTS idx_promotion_campaign_ident ON promotion.campaign (campaign_identifier) WHERE deleted_at IS NULL;

-- 2. campaign_budget
CREATE TABLE IF NOT EXISTS promotion.campaign_budget (
    id text PRIMARY KEY,
    campaign_id text NOT NULL REFERENCES promotion.campaign(id) ON DELETE CASCADE,
    type text NOT NULL, -- 'spend' or 'usage'
    "limit" numeric(20,6) NULL,
    used numeric(20,6) NOT NULL DEFAULT 0,
    currency_code text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- serves: GET /admin/campaigns/:id/budget
CREATE INDEX IF NOT EXISTS idx_promotion_budget_campaign_id ON promotion.campaign_budget (campaign_id);

-- 3. campaign_budget_usage
CREATE TABLE IF NOT EXISTS promotion.campaign_budget_usage (
    id text PRIMARY KEY,
    campaign_budget_id text NOT NULL REFERENCES promotion.campaign_budget(id) ON DELETE CASCADE,
    order_id text NULL,
    cart_id text NULL,
    amount numeric(20,6) NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- serves: GET /admin/campaigns/:id/usage
CREATE INDEX IF NOT EXISTS idx_promotion_budget_usage_budget_id ON promotion.campaign_budget_usage (campaign_budget_id);

-- 4. promotion
CREATE TABLE IF NOT EXISTS promotion.promotion (
    id text PRIMARY KEY,
    code text NOT NULL UNIQUE,
    type text NOT NULL DEFAULT 'standard', -- 'standard' or 'buyget'
    is_automatic boolean NOT NULL DEFAULT false,
    campaign_id text NULL REFERENCES promotion.campaign(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'draft', -- 'draft', 'active', 'inactive'
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/promotions?code=
CREATE INDEX IF NOT EXISTS idx_promotion_code ON promotion.promotion (code) WHERE deleted_at IS NULL;
-- serves: GET /store/promotions/automatic
CREATE INDEX IF NOT EXISTS idx_promotion_automatic ON promotion.promotion (is_automatic) WHERE is_automatic = true AND deleted_at IS NULL;

-- 5. application_method
CREATE TABLE IF NOT EXISTS promotion.application_method (
    id text PRIMARY KEY,
    promotion_id text NOT NULL REFERENCES promotion.promotion(id) ON DELETE CASCADE,
    type text NOT NULL, -- 'fixed' or 'percentage'
    target_type text NOT NULL, -- 'items', 'shipping_methods', 'order'
    allocation text NOT NULL DEFAULT 'across', -- 'each' or 'across'
    value numeric(20,6) NOT NULL,
    currency_code text NULL,
    max_quantity integer NULL,
    apply_to_quantity integer NULL,
    buy_rules_min_quantity integer NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NULL
);

-- serves: GET /store/promotions/:id/application-method
CREATE INDEX IF NOT EXISTS idx_promotion_app_method_promo_id ON promotion.application_method (promotion_id);

-- 6. promotion_rule
CREATE TABLE IF NOT EXISTS promotion.promotion_rule (
    id text PRIMARY KEY,
    promotion_id text NULL REFERENCES promotion.promotion(id) ON DELETE CASCADE,
    application_method_id text NULL REFERENCES promotion.application_method(id) ON DELETE CASCADE,
    rule_type text NOT NULL DEFAULT 'rules', -- 'rules', 'buy_rules', 'target_rules'
    attribute text NOT NULL,
    operator text NOT NULL, -- 'eq', 'ne', 'in', 'nin', 'gt', 'gte', 'lt', 'lte'
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- serves: GET /store/promotions/:id/rules
CREATE INDEX IF NOT EXISTS idx_promotion_rule_promo_id ON promotion.promotion_rule (promotion_id);
-- serves: GET /store/promotions/:id/app-rules
CREATE INDEX IF NOT EXISTS idx_promotion_rule_app_method_id ON promotion.promotion_rule (application_method_id);

-- 7. promotion_rule_value
CREATE TABLE IF NOT EXISTS promotion.promotion_rule_value (
    id text PRIMARY KEY,
    promotion_rule_id text NOT NULL REFERENCES promotion.promotion_rule(id) ON DELETE CASCADE,
    value text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- serves: JOIN promotion rule value resolution
CREATE INDEX IF NOT EXISTS idx_promotion_rule_value_rule_id ON promotion.promotion_rule_value (promotion_rule_id);
