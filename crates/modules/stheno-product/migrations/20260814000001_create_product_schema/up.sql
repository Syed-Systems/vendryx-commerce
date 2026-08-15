-- Phase: expand
-- Creates the product module schema and its 12 owned tables.

CREATE SCHEMA IF NOT EXISTS product;

-- 1. product
CREATE TABLE IF NOT EXISTS product.product (
    id text PRIMARY KEY,
    title text NOT NULL,
    handle text NOT NULL UNIQUE,
    subtitle text NULL,
    description text NULL,
    status text NOT NULL DEFAULT 'draft',
    thumbnail text NULL,
    weight numeric(20,6) NULL,
    length numeric(20,6) NULL,
    height numeric(20,6) NULL,
    width numeric(20,6) NULL,
    hs_code text NULL,
    origin_country text NULL,
    mid_code text NULL,
    material text NULL,
    discountable boolean NOT NULL DEFAULT true,
    external_id text NULL,
    type_id text NULL,
    collection_id text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/products?handle=
CREATE INDEX IF NOT EXISTS idx_product_handle ON product.product (handle) WHERE deleted_at IS NULL;
-- serves: GET /store/products?status=
CREATE INDEX IF NOT EXISTS idx_product_status ON product.product (status) WHERE deleted_at IS NULL;

-- 2. product_variant
CREATE TABLE IF NOT EXISTS product.product_variant (
    id text PRIMARY KEY,
    product_id text NOT NULL REFERENCES product.product(id) ON DELETE CASCADE,
    title text NOT NULL,
    sku text NULL UNIQUE,
    barcode text NULL,
    ean text NULL,
    upc text NULL,
    allow_backorder boolean NOT NULL DEFAULT false,
    manage_inventory boolean NOT NULL DEFAULT true,
    variant_rank integer NOT NULL DEFAULT 0,
    weight numeric(20,6) NULL,
    length numeric(20,6) NULL,
    height numeric(20,6) NULL,
    width numeric(20,6) NULL,
    hs_code text NULL,
    origin_country text NULL,
    mid_code text NULL,
    material text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/products/:id/variants
CREATE INDEX IF NOT EXISTS idx_product_variant_product_id ON product.product_variant (product_id) WHERE deleted_at IS NULL;

-- 3. product_option
CREATE TABLE IF NOT EXISTS product.product_option (
    id text PRIMARY KEY,
    product_id text NOT NULL REFERENCES product.product(id) ON DELETE CASCADE,
    title text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/products/:id/options
CREATE INDEX IF NOT EXISTS idx_product_option_product_id ON product.product_option (product_id) WHERE deleted_at IS NULL;

-- 4. product_option_value
CREATE TABLE IF NOT EXISTS product.product_option_value (
    id text PRIMARY KEY,
    option_id text NOT NULL REFERENCES product.product_option(id) ON DELETE CASCADE,
    value text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/products/options/:id/values
CREATE INDEX IF NOT EXISTS idx_product_option_value_option_id ON product.product_option_value (option_id) WHERE deleted_at IS NULL;

-- 5. product_product_option
CREATE TABLE IF NOT EXISTS product.product_product_option (
    id text PRIMARY KEY,
    product_id text NOT NULL REFERENCES product.product(id) ON DELETE CASCADE,
    option_id text NOT NULL REFERENCES product.product_option(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- 6. product_product_option_value
CREATE TABLE IF NOT EXISTS product.product_product_option_value (
    id text PRIMARY KEY,
    variant_id text NOT NULL REFERENCES product.product_variant(id) ON DELETE CASCADE,
    option_value_id text NOT NULL REFERENCES product.product_option_value(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: JOIN variant option resolution
CREATE INDEX IF NOT EXISTS idx_product_prov_variant ON product.product_product_option_value (variant_id);

-- 7. product_image
CREATE TABLE IF NOT EXISTS product.product_image (
    id text PRIMARY KEY,
    url text NOT NULL,
    rank integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- 8. product_variant_product_image
CREATE TABLE IF NOT EXISTS product.product_variant_product_image (
    id text PRIMARY KEY,
    variant_id text NOT NULL REFERENCES product.product_variant(id) ON DELETE CASCADE,
    image_id text NOT NULL REFERENCES product.product_image(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- 9. product_category
CREATE TABLE IF NOT EXISTS product.product_category (
    id text PRIMARY KEY,
    name text NOT NULL,
    handle text NOT NULL,
    parent_category_id text NULL REFERENCES product.product_category(id) ON DELETE SET NULL,
    rank integer NOT NULL DEFAULT 0,
    is_active boolean NOT NULL DEFAULT true,
    is_internal boolean NOT NULL DEFAULT false,
    mpath text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/product-categories?handle=
CREATE INDEX IF NOT EXISTS idx_product_category_handle ON product.product_category (handle) WHERE deleted_at IS NULL;
-- serves: GET /store/product-categories subtree mpath query
CREATE INDEX IF NOT EXISTS idx_product_category_mpath ON product.product_category (mpath text_pattern_ops) WHERE deleted_at IS NULL;

-- 10. product_collection
CREATE TABLE IF NOT EXISTS product.product_collection (
    id text PRIMARY KEY,
    title text NOT NULL,
    handle text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- 11. product_tag
CREATE TABLE IF NOT EXISTS product.product_tag (
    id text PRIMARY KEY,
    value text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- 12. product_type
CREATE TABLE IF NOT EXISTS product.product_type (
    id text PRIMARY KEY,
    value text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);
