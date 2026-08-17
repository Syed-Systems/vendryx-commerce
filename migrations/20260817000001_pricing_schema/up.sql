-- Phase: expand
-- Creates the pricing module schema (DATA_MODEL §2, P12-02).
--
-- Money is numeric(20,6) throughout, never float and never integer minor units
-- (ADR-006). A currency with 3 decimal places (KWD) or 0 (JPY) must round-trip
-- exactly, and the price calculation path is the last place to discover it did
-- not.

CREATE SCHEMA IF NOT EXISTS pricing;

-- 1. price_set -- the anchor a variant's prices hang from.
CREATE TABLE IF NOT EXISTS pricing.price_set (
    id         text PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- 2. price_list -- a named override set with an optional active window.
CREATE TABLE IF NOT EXISTS pricing.price_list (
    id          text PRIMARY KEY,
    title       text NOT NULL,
    description text NULL,
    status      text NOT NULL DEFAULT 'draft',
    type        text NOT NULL DEFAULT 'sale',
    starts_at   timestamptz NULL,
    ends_at     timestamptz NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL,
    -- A window that ends before it starts would silently never apply.
    CONSTRAINT price_list_window_is_ordered
        CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at <= ends_at)
);

-- serves: resolving which price lists are active at calculation time
CREATE INDEX IF NOT EXISTS idx_price_list_status
    ON pricing.price_list (status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_price_list_window
    ON pricing.price_list (starts_at, ends_at) WHERE deleted_at IS NULL;

-- 3. price
CREATE TABLE IF NOT EXISTS pricing.price (
    id            text PRIMARY KEY,
    price_set_id  text NOT NULL REFERENCES pricing.price_set (id) ON DELETE CASCADE,
    price_list_id text NULL REFERENCES pricing.price_list (id) ON DELETE CASCADE,
    currency_code text NOT NULL,
    amount        numeric(20,6) NOT NULL,
    min_quantity  integer NULL,
    max_quantity  integer NULL,
    -- Denormalised rule count: specificity ranking orders by it on every
    -- calculation, and counting rules per candidate price would be an N+1.
    rules_count   integer NOT NULL DEFAULT 0,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz NULL,
    CONSTRAINT price_currency_is_lowercase CHECK (currency_code = lower(currency_code)),
    CONSTRAINT price_quantity_tier_is_ordered
        CHECK (min_quantity IS NULL OR max_quantity IS NULL OR min_quantity <= max_quantity)
);

-- serves: the price calculation hot path, which selects by set then filters
CREATE INDEX IF NOT EXISTS idx_price_price_set_id
    ON pricing.price (price_set_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_price_price_list_id
    ON pricing.price (price_list_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_price_set_currency
    ON pricing.price (price_set_id, currency_code) WHERE deleted_at IS NULL;

-- 4. price_rule -- the attributes that make one price more specific than another.
CREATE TABLE IF NOT EXISTS pricing.price_rule (
    id         text PRIMARY KEY,
    price_id   text NOT NULL REFERENCES pricing.price (id) ON DELETE CASCADE,
    attribute  text NOT NULL,
    value      text NOT NULL,
    priority   integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_price_rule_price_id
    ON pricing.price_rule (price_id) WHERE deleted_at IS NULL;
-- One value per attribute per price: two rules on the same attribute would make
-- specificity ranking depend on row order.
CREATE UNIQUE INDEX IF NOT EXISTS idx_price_rule_price_attribute
    ON pricing.price_rule (price_id, attribute) WHERE deleted_at IS NULL;

-- 5. price_list_rule
CREATE TABLE IF NOT EXISTS pricing.price_list_rule (
    id            text PRIMARY KEY,
    price_list_id text NOT NULL REFERENCES pricing.price_list (id) ON DELETE CASCADE,
    attribute     text NOT NULL,
    value         text NOT NULL,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_price_list_rule_list_id
    ON pricing.price_list_rule (price_list_id) WHERE deleted_at IS NULL;

-- 6. price_preference -- tax-inclusivity per attribute/value pair.
CREATE TABLE IF NOT EXISTS pricing.price_preference (
    id               text PRIMARY KEY,
    attribute        text NOT NULL,
    value            text NOT NULL,
    is_tax_inclusive boolean NOT NULL DEFAULT false,
    created_at       timestamptz NOT NULL DEFAULT now(),
    updated_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at       timestamptz NULL
);

-- One preference per pair, or tax inclusivity becomes order-dependent.
CREATE UNIQUE INDEX IF NOT EXISTS idx_price_preference_attribute_value
    ON pricing.price_preference (attribute, value) WHERE deleted_at IS NULL;
