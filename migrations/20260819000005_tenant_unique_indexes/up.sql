-- ADR-012: natural-key uniqueness is per tenant, not global.
--
-- Adding tenant_id and RLS scopes what a tenant can SEE. It does not change what the
-- database considers a duplicate. A unique index on `product.handle` is still global, so
-- the second tenant to create a product called "t-shirt" gets a unique violation on a row
-- it cannot even read — an error that is impossible to diagnose from the application.
--
-- The same applies to logins (`user.email`, `auth_identity.entity_id`), promo codes,
-- SKUs, role names, locale codes and the reporting rollups. Every one of these is a
-- natural key that two tenants may legitimately share.
--
-- Written out explicitly rather than looped: unlike the column addition, each index needs
-- a judgement about whether its key is natural (rewrite) or a globally-unique ULID
-- (leave alone). Only the natural ones are here.

-- Identity and access -------------------------------------------------------
ALTER TABLE auth.auth_identity DROP CONSTRAINT IF EXISTS idx_auth_identity_provider_entity;
DROP INDEX IF EXISTS auth.idx_auth_identity_provider_entity;
CREATE UNIQUE INDEX idx_auth_identity_provider_entity
    ON auth.auth_identity (tenant_id, provider_id, entity_id) WHERE deleted_at IS NULL;

ALTER TABLE "user"."user" DROP CONSTRAINT IF EXISTS idx_user_email;
DROP INDEX IF EXISTS "user".idx_user_email;
CREATE UNIQUE INDEX idx_user_email
    ON "user"."user" (tenant_id, email) WHERE deleted_at IS NULL;

ALTER TABLE rbac."role" DROP CONSTRAINT IF EXISTS idx_role_name;
DROP INDEX IF EXISTS rbac.idx_role_name;
CREATE UNIQUE INDEX idx_role_name
    ON rbac."role" (tenant_id, name) WHERE deleted_at IS NULL;

-- Catalog -------------------------------------------------------------------
ALTER TABLE product.product DROP CONSTRAINT IF EXISTS product_handle_key;
CREATE UNIQUE INDEX IF NOT EXISTS product_handle_key
    ON product.product (tenant_id, handle);

ALTER TABLE product.product_collection DROP CONSTRAINT IF EXISTS product_collection_handle_key;
CREATE UNIQUE INDEX IF NOT EXISTS product_collection_handle_key
    ON product.product_collection (tenant_id, handle);

ALTER TABLE product.product_variant DROP CONSTRAINT IF EXISTS product_variant_sku_key;
CREATE UNIQUE INDEX IF NOT EXISTS product_variant_sku_key
    ON product.product_variant (tenant_id, sku);

ALTER TABLE product.product_tag DROP CONSTRAINT IF EXISTS product_tag_value_key;
CREATE UNIQUE INDEX IF NOT EXISTS product_tag_value_key
    ON product.product_tag (tenant_id, value);

ALTER TABLE product.product_type DROP CONSTRAINT IF EXISTS product_type_value_key;
CREATE UNIQUE INDEX IF NOT EXISTS product_type_value_key
    ON product.product_type (tenant_id, value);

ALTER TABLE product.product_variant DROP CONSTRAINT IF EXISTS idx_variant_product_option_hash;
DROP INDEX IF EXISTS product.idx_variant_product_option_hash;
CREATE UNIQUE INDEX idx_variant_product_option_hash
    ON product.product_variant (tenant_id, product_id, option_values_hash)
    WHERE deleted_at IS NULL;

-- Commerce ------------------------------------------------------------------
ALTER TABLE promotion.promotion DROP CONSTRAINT IF EXISTS promotion_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS promotion_code_key
    ON promotion.promotion (tenant_id, code);

ALTER TABLE "order".return_reason DROP CONSTRAINT IF EXISTS return_reason_value_key;
CREATE UNIQUE INDEX IF NOT EXISTS return_reason_value_key
    ON "order".return_reason (tenant_id, value);

ALTER TABLE pricing.price_preference DROP CONSTRAINT IF EXISTS idx_price_preference_attribute_value;
DROP INDEX IF EXISTS pricing.idx_price_preference_attribute_value;
CREATE UNIQUE INDEX idx_price_preference_attribute_value
    ON pricing.price_preference (tenant_id, attribute, value) WHERE deleted_at IS NULL;

ALTER TABLE inventory.inventory_level DROP CONSTRAINT IF EXISTS unq_inventory_level_item_location;
DROP INDEX IF EXISTS inventory.unq_inventory_level_item_location;
CREATE UNIQUE INDEX unq_inventory_level_item_location
    ON inventory.inventory_level (tenant_id, inventory_item_id, location_id);

-- Configuration and platform ops --------------------------------------------
ALTER TABLE translation.locale DROP CONSTRAINT IF EXISTS locale_code_key;
CREATE UNIQUE INDEX IF NOT EXISTS locale_code_key
    ON translation.locale (tenant_id, code);

ALTER TABLE settings.store_setting DROP CONSTRAINT IF EXISTS store_setting_key_key;
CREATE UNIQUE INDEX IF NOT EXISTS store_setting_key_key
    ON settings.store_setting (tenant_id, key);

-- Primary keys whose key is natural rather than a ULID ----------------------
ALTER TABLE caching.cache_entry DROP CONSTRAINT IF EXISTS cache_entry_pkey;
ALTER TABLE caching.cache_entry ADD PRIMARY KEY (tenant_id, key);

ALTER TABLE reporting.daily_rollup DROP CONSTRAINT IF EXISTS daily_rollup_pkey;
ALTER TABLE reporting.daily_rollup ADD PRIMARY KEY (tenant_id, date, currency);

ALTER TABLE reporting.daily_product_rollup DROP CONSTRAINT IF EXISTS daily_product_rollup_pkey;
ALTER TABLE reporting.daily_product_rollup ADD PRIMARY KEY (tenant_id, date, product_id);
