-- Stock Location Schema
CREATE SCHEMA IF NOT EXISTS stock_location;

CREATE TABLE stock_location.stock_location (
    id text PRIMARY KEY,
    name text NOT NULL,
    address_id text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/stock-locations?name=
CREATE INDEX idx_stock_location_name ON stock_location.stock_location(name) WHERE deleted_at IS NULL;

CREATE TABLE stock_location.stock_location_address (
    id text PRIMARY KEY,
    address_1 text NOT NULL,
    address_2 text NULL,
    city text NULL,
    country_code text NOT NULL,
    postal_code text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- Sales Channel Schema
CREATE SCHEMA IF NOT EXISTS sales_channel;

CREATE TABLE sales_channel.sales_channel (
    id text PRIMARY KEY,
    name text NOT NULL,
    description text NULL,
    is_disabled boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/sales-channels?name=
CREATE INDEX idx_sales_channel_name ON sales_channel.sales_channel(name) WHERE deleted_at IS NULL;

-- Tax Schema
CREATE SCHEMA IF NOT EXISTS tax;

CREATE TABLE tax.tax_region (
    id text PRIMARY KEY,
    country_code text NOT NULL,
    province_code text NULL,
    parent_id text NULL,
    provider_id text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/tax-regions?country_code=&province_code=
CREATE INDEX idx_tax_region_country_province ON tax.tax_region(country_code, province_code) WHERE deleted_at IS NULL;
-- serves: Tax resolution parent lookup
CREATE INDEX idx_tax_region_parent ON tax.tax_region(parent_id) WHERE deleted_at IS NULL;

CREATE TABLE tax.tax_rate (
    id text PRIMARY KEY,
    tax_region_id text NOT NULL,
    rate numeric(20, 6) NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    is_default boolean NOT NULL DEFAULT false,
    is_combinable boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/tax-rates?tax_region_id=
CREATE INDEX idx_tax_rate_region ON tax.tax_rate(tax_region_id) WHERE deleted_at IS NULL;

CREATE TABLE tax.tax_rate_rule (
    id text PRIMARY KEY,
    tax_rate_id text NOT NULL,
    reference text NOT NULL,
    reference_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Tax resolution rate rules lookup
CREATE INDEX idx_tax_rate_rule_rate ON tax.tax_rate_rule(tax_rate_id) WHERE deleted_at IS NULL;

CREATE TABLE tax.tax_provider (
    id text PRIMARY KEY,
    is_installed boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- Store Schema
CREATE SCHEMA IF NOT EXISTS store;

CREATE TABLE store.store (
    id text PRIMARY KEY,
    name text NOT NULL,
    default_sales_channel_id text NULL,
    default_region_id text NULL,
    default_location_id text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE store.store_currency (
    id text PRIMARY KEY,
    store_id text NOT NULL,
    currency_code text NOT NULL,
    is_default boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/stores/:id/currencies
CREATE INDEX idx_store_currency_store ON store.store_currency(store_id) WHERE deleted_at IS NULL;

CREATE TABLE store.store_locale (
    id text PRIMARY KEY,
    store_id text NOT NULL,
    locale text NOT NULL,
    is_default boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/stores/:id/locales
CREATE INDEX idx_store_locale_store ON store.store_locale(store_id) WHERE deleted_at IS NULL;
