DROP INDEX IF EXISTS product.idx_variant_product_option_hash;
ALTER TABLE product.product_variant DROP COLUMN IF EXISTS option_values_hash;
