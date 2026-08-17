-- Phase: expand
-- Creates the cart module schema and its 9 owned tables.

CREATE SCHEMA IF NOT EXISTS cart;

-- 1. address
CREATE TABLE IF NOT EXISTS cart.address (
    id text PRIMARY KEY,
    customer_id text NULL,
    company text NULL,
    first_name text NULL,
    last_name text NULL,
    address_1 text NULL,
    address_2 text NULL,
    city text NULL,
    country_code text NULL,
    province text NULL,
    postal_code text NULL,
    phone text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/carts/:id/shipping-address
CREATE INDEX IF NOT EXISTS idx_cart_address_customer_id ON cart.address (customer_id) WHERE deleted_at IS NULL;

-- 2. cart
CREATE TABLE IF NOT EXISTS cart.cart (
    id text PRIMARY KEY,
    region_id text NULL,
    customer_id text NULL,
    sales_channel_id text NULL,
    email text NULL,
    currency_code text NOT NULL,
    shipping_address_id text NULL REFERENCES cart.address(id) ON DELETE SET NULL,
    billing_address_id text NULL REFERENCES cart.address(id) ON DELETE SET NULL,
    cart_secret text NOT NULL,
    completed_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/carts?customer_id=
CREATE INDEX IF NOT EXISTS idx_cart_customer_id ON cart.cart (customer_id) WHERE deleted_at IS NULL;
-- serves: GET /admin/carts active carts filter
CREATE INDEX IF NOT EXISTS idx_cart_active ON cart.cart (completed_at) WHERE completed_at IS NULL AND deleted_at IS NULL;

-- 3. line_item
CREATE TABLE IF NOT EXISTS cart.line_item (
    id text PRIMARY KEY,
    cart_id text NOT NULL REFERENCES cart.cart(id) ON DELETE CASCADE,
    title text NOT NULL,
    subtitle text NULL,
    thumbnail text NULL,
    quantity integer NOT NULL DEFAULT 1,
    variant_id text NULL,
    product_id text NULL,
    product_title text NULL,
    product_description text NULL,
    product_subtitle text NULL,
    product_type text NULL,
    product_collection text NULL,
    product_handle text NULL,
    variant_sku text NULL,
    variant_barcode text NULL,
    variant_title text NULL,
    variant_option_values jsonb NULL,
    requires_shipping boolean NOT NULL DEFAULT true,
    is_discountable boolean NOT NULL DEFAULT true,
    is_tax_inclusive boolean NOT NULL DEFAULT false,
    is_custom_price boolean NOT NULL DEFAULT false,
    unit_price numeric(20,6) NOT NULL,
    compare_at_unit_price numeric(20,6) NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/carts/:id/line-items
CREATE INDEX IF NOT EXISTS idx_cart_line_item_cart_id ON cart.line_item (cart_id) WHERE deleted_at IS NULL;

-- 4. line_item_adjustment
CREATE TABLE IF NOT EXISTS cart.line_item_adjustment (
    id text PRIMARY KEY,
    item_id text NOT NULL REFERENCES cart.line_item(id) ON DELETE CASCADE,
    amount numeric(20,6) NOT NULL,
    promotion_id text NULL,
    code text NULL,
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /store/carts/:id/line-item-adjustments
CREATE INDEX IF NOT EXISTS idx_cart_line_item_adj_item_id ON cart.line_item_adjustment (item_id);

-- 5. line_item_tax_line
CREATE TABLE IF NOT EXISTS cart.line_item_tax_line (
    id text PRIMARY KEY,
    item_id text NOT NULL REFERENCES cart.line_item(id) ON DELETE CASCADE,
    rate numeric(10,4) NOT NULL,
    code text NOT NULL,
    tax_rate_id text NULL,
    provider_id text NULL,
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /store/carts/:id/line-item-taxes
CREATE INDEX IF NOT EXISTS idx_cart_line_item_tax_item_id ON cart.line_item_tax_line (item_id);

-- 6. shipping_method
CREATE TABLE IF NOT EXISTS cart.shipping_method (
    id text PRIMARY KEY,
    cart_id text NOT NULL REFERENCES cart.cart(id) ON DELETE CASCADE,
    name text NOT NULL,
    amount numeric(20,6) NOT NULL,
    is_tax_inclusive boolean NOT NULL DEFAULT false,
    shipping_option_id text NULL,
    data jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/carts/:id/shipping-methods
CREATE INDEX IF NOT EXISTS idx_cart_shipping_method_cart_id ON cart.shipping_method (cart_id);

-- 7. shipping_method_adjustment
CREATE TABLE IF NOT EXISTS cart.shipping_method_adjustment (
    id text PRIMARY KEY,
    shipping_method_id text NOT NULL REFERENCES cart.shipping_method(id) ON DELETE CASCADE,
    amount numeric(20,6) NOT NULL,
    promotion_id text NULL,
    code text NULL,
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /store/carts/:id/shipping-adjustments
CREATE INDEX IF NOT EXISTS idx_cart_sm_adj_sm_id ON cart.shipping_method_adjustment (shipping_method_id);

-- 8. shipping_method_tax_line
CREATE TABLE IF NOT EXISTS cart.shipping_method_tax_line (
    id text PRIMARY KEY,
    shipping_method_id text NOT NULL REFERENCES cart.shipping_method(id) ON DELETE CASCADE,
    rate numeric(10,4) NOT NULL,
    code text NOT NULL,
    tax_rate_id text NULL,
    provider_id text NULL,
    description text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /store/carts/:id/shipping-taxes
CREATE INDEX IF NOT EXISTS idx_cart_sm_tax_sm_id ON cart.shipping_method_tax_line (shipping_method_id);

-- 9. credit_line
CREATE TABLE IF NOT EXISTS cart.credit_line (
    id text PRIMARY KEY,
    cart_id text NOT NULL REFERENCES cart.cart(id) ON DELETE CASCADE,
    amount numeric(20,6) NOT NULL,
    reference text NOT NULL,
    reference_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/carts/:id/credit-lines
CREATE INDEX IF NOT EXISTS idx_cart_credit_line_cart_id ON cart.credit_line (cart_id);
