-- Phase 14: Cross-module link tables in the `links` schema (DATA_MODEL.md §3.1)
CREATE SCHEMA IF NOT EXISTS links;

-- 1. link_product_variant_price_set
CREATE TABLE IF NOT EXISTS links.link_product_variant_price_set (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_product_variant_price_set ON links.link_product_variant_price_set (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_variant_price_set_left ON links.link_product_variant_price_set (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_variant_price_set_right ON links.link_product_variant_price_set (right_id) WHERE deleted_at IS NULL;

-- 2. link_product_variant_inventory_item
CREATE TABLE IF NOT EXISTS links.link_product_variant_inventory_item (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_product_variant_inventory_item ON links.link_product_variant_inventory_item (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_variant_inventory_item_left ON links.link_product_variant_inventory_item (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_variant_inventory_item_right ON links.link_product_variant_inventory_item (right_id) WHERE deleted_at IS NULL;

-- 3. link_product_sales_channel
CREATE TABLE IF NOT EXISTS links.link_product_sales_channel (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_product_sales_channel ON links.link_product_sales_channel (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_sales_channel_left ON links.link_product_sales_channel (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_sales_channel_right ON links.link_product_sales_channel (right_id) WHERE deleted_at IS NULL;

-- 4. link_product_shipping_profile
CREATE TABLE IF NOT EXISTS links.link_product_shipping_profile (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_product_shipping_profile ON links.link_product_shipping_profile (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_shipping_profile_left ON links.link_product_shipping_profile (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_product_shipping_profile_right ON links.link_product_shipping_profile (right_id) WHERE deleted_at IS NULL;

-- 5. link_shipping_option_price_set
CREATE TABLE IF NOT EXISTS links.link_shipping_option_price_set (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_shipping_option_price_set ON links.link_shipping_option_price_set (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_shipping_option_price_set_left ON links.link_shipping_option_price_set (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_shipping_option_price_set_right ON links.link_shipping_option_price_set (right_id) WHERE deleted_at IS NULL;

-- 6. link_cart_payment_collection
CREATE TABLE IF NOT EXISTS links.link_cart_payment_collection (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_cart_payment_collection ON links.link_cart_payment_collection (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_cart_payment_collection_left ON links.link_cart_payment_collection (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_cart_payment_collection_right ON links.link_cart_payment_collection (right_id) WHERE deleted_at IS NULL;

-- 7. link_cart_promotion
CREATE TABLE IF NOT EXISTS links.link_cart_promotion (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_cart_promotion ON links.link_cart_promotion (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_cart_promotion_left ON links.link_cart_promotion (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_cart_promotion_right ON links.link_cart_promotion (right_id) WHERE deleted_at IS NULL;

-- 8. link_order_cart
CREATE TABLE IF NOT EXISTS links.link_order_cart (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_cart ON links.link_order_cart (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_cart_left ON links.link_order_cart (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_cart_right ON links.link_order_cart (right_id) WHERE deleted_at IS NULL;

-- 9. link_order_payment_collection
CREATE TABLE IF NOT EXISTS links.link_order_payment_collection (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_payment_collection ON links.link_order_payment_collection (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_payment_collection_left ON links.link_order_payment_collection (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_payment_collection_right ON links.link_order_payment_collection (right_id) WHERE deleted_at IS NULL;

-- 10. link_order_claim_payment_collection
CREATE TABLE IF NOT EXISTS links.link_order_claim_payment_collection (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_claim_payment_collection ON links.link_order_claim_payment_collection (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_claim_payment_collection_left ON links.link_order_claim_payment_collection (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_claim_payment_collection_right ON links.link_order_claim_payment_collection (right_id) WHERE deleted_at IS NULL;

-- 11. link_order_exchange_payment_collection
CREATE TABLE IF NOT EXISTS links.link_order_exchange_payment_collection (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_exchange_payment_collection ON links.link_order_exchange_payment_collection (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_exchange_payment_collection_left ON links.link_order_exchange_payment_collection (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_exchange_payment_collection_right ON links.link_order_exchange_payment_collection (right_id) WHERE deleted_at IS NULL;

-- 12. link_order_promotion
CREATE TABLE IF NOT EXISTS links.link_order_promotion (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_promotion ON links.link_order_promotion (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_promotion_left ON links.link_order_promotion (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_promotion_right ON links.link_order_promotion (right_id) WHERE deleted_at IS NULL;

-- 13. link_order_fulfillment
CREATE TABLE IF NOT EXISTS links.link_order_fulfillment (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_fulfillment ON links.link_order_fulfillment (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_fulfillment_left ON links.link_order_fulfillment (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_fulfillment_right ON links.link_order_fulfillment (right_id) WHERE deleted_at IS NULL;

-- 14. link_order_return_fulfillment
CREATE TABLE IF NOT EXISTS links.link_order_return_fulfillment (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_order_return_fulfillment ON links.link_order_return_fulfillment (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_return_fulfillment_left ON links.link_order_return_fulfillment (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_order_return_fulfillment_right ON links.link_order_return_fulfillment (right_id) WHERE deleted_at IS NULL;

-- 15. link_customer_account_holder
CREATE TABLE IF NOT EXISTS links.link_customer_account_holder (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_customer_account_holder ON links.link_customer_account_holder (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_customer_account_holder_left ON links.link_customer_account_holder (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_customer_account_holder_right ON links.link_customer_account_holder (right_id) WHERE deleted_at IS NULL;

-- 16. link_region_payment_provider
CREATE TABLE IF NOT EXISTS links.link_region_payment_provider (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_region_payment_provider ON links.link_region_payment_provider (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_region_payment_provider_left ON links.link_region_payment_provider (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_region_payment_provider_right ON links.link_region_payment_provider (right_id) WHERE deleted_at IS NULL;

-- 17. link_sales_channel_location
CREATE TABLE IF NOT EXISTS links.link_sales_channel_location (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_sales_channel_location ON links.link_sales_channel_location (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_sales_channel_location_left ON links.link_sales_channel_location (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_sales_channel_location_right ON links.link_sales_channel_location (right_id) WHERE deleted_at IS NULL;

-- 18. link_fulfillment_set_location
CREATE TABLE IF NOT EXISTS links.link_fulfillment_set_location (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_fulfillment_set_location ON links.link_fulfillment_set_location (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_fulfillment_set_location_left ON links.link_fulfillment_set_location (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_fulfillment_set_location_right ON links.link_fulfillment_set_location (right_id) WHERE deleted_at IS NULL;

-- 19. link_fulfillment_provider_location
CREATE TABLE IF NOT EXISTS links.link_fulfillment_provider_location (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_fulfillment_provider_location ON links.link_fulfillment_provider_location (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_fulfillment_provider_location_left ON links.link_fulfillment_provider_location (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_fulfillment_provider_location_right ON links.link_fulfillment_provider_location (right_id) WHERE deleted_at IS NULL;

-- 20. link_publishable_api_key_sales_channel
CREATE TABLE IF NOT EXISTS links.link_publishable_api_key_sales_channel (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_publishable_api_key_sales_channel ON links.link_publishable_api_key_sales_channel (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_publishable_api_key_sales_channel_left ON links.link_publishable_api_key_sales_channel (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_publishable_api_key_sales_channel_right ON links.link_publishable_api_key_sales_channel (right_id) WHERE deleted_at IS NULL;

-- 21. link_user_rbac_role
CREATE TABLE IF NOT EXISTS links.link_user_rbac_role (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_user_rbac_role ON links.link_user_rbac_role (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_user_rbac_role_left ON links.link_user_rbac_role (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_user_rbac_role_right ON links.link_user_rbac_role (right_id) WHERE deleted_at IS NULL;

-- 22. link_invite_rbac_role
CREATE TABLE IF NOT EXISTS links.link_invite_rbac_role (
    id text PRIMARY KEY,
    left_id text NOT NULL,
    right_id text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_link_invite_rbac_role ON links.link_invite_rbac_role (left_id, right_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_invite_rbac_role_left ON links.link_invite_rbac_role (left_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_link_invite_rbac_role_right ON links.link_invite_rbac_role (right_id) WHERE deleted_at IS NULL;
