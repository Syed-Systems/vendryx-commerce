-- Migration for stheno-fulfillment schema (DATA_MODEL §2.6, TASKS P5-07)
CREATE SCHEMA IF NOT EXISTS "fulfillment";

-- 1. Address
CREATE TABLE IF NOT EXISTS "fulfillment"."address" (
    id VARCHAR(255) PRIMARY KEY,
    customer_id VARCHAR(255),
    company VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    address_1 VARCHAR(255),
    address_2 VARCHAR(255),
    city VARCHAR(255),
    country_code VARCHAR(10),
    province VARCHAR(255),
    postal_code VARCHAR(50),
    phone VARCHAR(50),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 2. FulfillmentProvider
CREATE TABLE IF NOT EXISTS "fulfillment"."fulfillment_provider" (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

INSERT INTO "fulfillment"."fulfillment_provider" (id, name)
VALUES ('manual', 'Manual Fulfillment Provider')
ON CONFLICT (id) DO NOTHING;

-- 3. ShippingProfile
CREATE TABLE IF NOT EXISTS "fulfillment"."shipping_profile" (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL DEFAULT 'default',
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 4. ShippingOptionType
CREATE TABLE IF NOT EXISTS "fulfillment"."shipping_option_type" (
    id VARCHAR(255) PRIMARY KEY,
    label VARCHAR(255) NOT NULL,
    description TEXT,
    code VARCHAR(100) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 5. FulfillmentSet
CREATE TABLE IF NOT EXISTS "fulfillment"."fulfillment_set" (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL DEFAULT 'shipping',
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 6. ServiceZone
CREATE TABLE IF NOT EXISTS "fulfillment"."service_zone" (
    id VARCHAR(255) PRIMARY KEY,
    fulfillment_set_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."fulfillment_set"(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/fulfillment-sets/{id}/service-zones
CREATE INDEX IF NOT EXISTS idx_service_zone_fulfillment_set ON "fulfillment"."service_zone" (fulfillment_set_id) WHERE deleted_at IS NULL;

-- 7. GeoZone
CREATE TABLE IF NOT EXISTS "fulfillment"."geo_zone" (
    id VARCHAR(255) PRIMARY KEY,
    service_zone_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."service_zone"(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL DEFAULT 'country',
    country_code VARCHAR(10) NOT NULL,
    province_code VARCHAR(50),
    city VARCHAR(255),
    postal_expression JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/service-zones/{id}/geo-zones
CREATE INDEX IF NOT EXISTS idx_geo_zone_service_zone ON "fulfillment"."geo_zone" (service_zone_id);
-- serves: POST /store/shipping-options/calculate (geo match)
CREATE INDEX IF NOT EXISTS idx_geo_zone_country_province ON "fulfillment"."geo_zone" (country_code, province_code);

-- 8. ShippingOption
CREATE TABLE IF NOT EXISTS "fulfillment"."shipping_option" (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price_type VARCHAR(50) NOT NULL DEFAULT 'flat',
    service_zone_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."service_zone"(id) ON DELETE CASCADE,
    shipping_profile_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."shipping_profile"(id) ON DELETE CASCADE,
    provider_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."fulfillment_provider"(id) ON DELETE CASCADE,
    shipping_option_type_id VARCHAR(255) REFERENCES "fulfillment"."shipping_option_type"(id) ON DELETE SET NULL,
    data JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /store/shipping-options?service_zone_id=
CREATE INDEX IF NOT EXISTS idx_shipping_option_service_zone ON "fulfillment"."shipping_option" (service_zone_id) WHERE deleted_at IS NULL;

-- 9. ShippingOptionRule
CREATE TABLE IF NOT EXISTS "fulfillment"."shipping_option_rule" (
    id VARCHAR(255) PRIMARY KEY,
    shipping_option_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."shipping_option"(id) ON DELETE CASCADE,
    attribute VARCHAR(100) NOT NULL,
    operator VARCHAR(20) NOT NULL,
    value JSONB NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /store/shipping-options (eligibility filter)
CREATE INDEX IF NOT EXISTS idx_shipping_option_rule_option ON "fulfillment"."shipping_option_rule" (shipping_option_id);

-- 10. Fulfillment
CREATE TABLE IF NOT EXISTS "fulfillment"."fulfillment" (
    id VARCHAR(255) PRIMARY KEY,
    location_id VARCHAR(255),
    provider_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."fulfillment_provider"(id) ON DELETE CASCADE,
    shipping_option_id VARCHAR(255) REFERENCES "fulfillment"."shipping_option"(id) ON DELETE SET NULL,
    delivery_address_id VARCHAR(255) REFERENCES "fulfillment"."address"(id) ON DELETE SET NULL,
    packed_at TIMESTAMPTZ,
    shipped_at TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    data JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/fulfillments?provider_id=
CREATE INDEX IF NOT EXISTS idx_fulfillment_provider ON "fulfillment"."fulfillment" (provider_id) WHERE deleted_at IS NULL;

-- 11. FulfillmentItem
CREATE TABLE IF NOT EXISTS "fulfillment"."fulfillment_item" (
    id VARCHAR(255) PRIMARY KEY,
    fulfillment_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."fulfillment"(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    sku VARCHAR(255),
    barcode VARCHAR(255),
    line_item_id VARCHAR(255) NOT NULL,
    inventory_item_id VARCHAR(255),
    quantity INT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/fulfillments/{id}/items
CREATE INDEX IF NOT EXISTS idx_fulfillment_item_fulfillment ON "fulfillment"."fulfillment_item" (fulfillment_id);

-- 12. FulfillmentLabel
CREATE TABLE IF NOT EXISTS "fulfillment"."fulfillment_label" (
    id VARCHAR(255) PRIMARY KEY,
    fulfillment_id VARCHAR(255) NOT NULL REFERENCES "fulfillment"."fulfillment"(id) ON DELETE CASCADE,
    tracking_number VARCHAR(255) NOT NULL,
    tracking_url VARCHAR(1024),
    label_url VARCHAR(1024),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/fulfillments/{id}/labels
CREATE INDEX IF NOT EXISTS idx_fulfillment_label_fulfillment ON "fulfillment"."fulfillment_label" (fulfillment_id) WHERE deleted_at IS NULL;
