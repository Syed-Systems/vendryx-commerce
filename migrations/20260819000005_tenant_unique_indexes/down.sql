-- Restores global uniqueness. Only safe while a single tenant has data.
DROP INDEX IF EXISTS auth.idx_auth_identity_provider_entity;
CREATE UNIQUE INDEX idx_auth_identity_provider_entity
    ON auth.auth_identity (provider_id, entity_id) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS "user".idx_user_email;
CREATE UNIQUE INDEX idx_user_email ON "user"."user" (email) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS rbac.idx_role_name;
CREATE UNIQUE INDEX idx_role_name ON rbac."role" (name) WHERE deleted_at IS NULL;
DROP INDEX IF EXISTS product.product_handle_key;
DROP INDEX IF EXISTS product.product_collection_handle_key;
DROP INDEX IF EXISTS product.product_variant_sku_key;
DROP INDEX IF EXISTS product.product_tag_value_key;
DROP INDEX IF EXISTS product.product_type_value_key;
DROP INDEX IF EXISTS product.idx_variant_product_option_hash;
DROP INDEX IF EXISTS promotion.promotion_code_key;
DROP INDEX IF EXISTS "order".return_reason_value_key;
DROP INDEX IF EXISTS pricing.idx_price_preference_attribute_value;
DROP INDEX IF EXISTS inventory.unq_inventory_level_item_location;
DROP INDEX IF EXISTS translation.locale_code_key;
DROP INDEX IF EXISTS settings.store_setting_key_key;
ALTER TABLE caching.cache_entry DROP CONSTRAINT IF EXISTS cache_entry_pkey;
ALTER TABLE caching.cache_entry ADD PRIMARY KEY (key);
ALTER TABLE reporting.daily_rollup DROP CONSTRAINT IF EXISTS daily_rollup_pkey;
ALTER TABLE reporting.daily_rollup ADD PRIMARY KEY (date, currency);
ALTER TABLE reporting.daily_product_rollup DROP CONSTRAINT IF EXISTS daily_product_rollup_pkey;
ALTER TABLE reporting.daily_product_rollup ADD PRIMARY KEY (date, product_id);
