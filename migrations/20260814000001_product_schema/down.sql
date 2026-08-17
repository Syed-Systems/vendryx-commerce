-- Phase: contract
-- Drops the product module tables and schema.

DROP TABLE IF EXISTS product.product_type CASCADE;
DROP TABLE IF EXISTS product.product_tag CASCADE;
DROP TABLE IF EXISTS product.product_collection CASCADE;
DROP TABLE IF EXISTS product.product_category CASCADE;
DROP TABLE IF EXISTS product.product_variant_product_image CASCADE;
DROP TABLE IF EXISTS product.product_image CASCADE;
DROP TABLE IF EXISTS product.product_product_option_value CASCADE;
DROP TABLE IF EXISTS product.product_product_option CASCADE;
DROP TABLE IF EXISTS product.product_option_value CASCADE;
DROP TABLE IF EXISTS product.product_option CASCADE;
DROP TABLE IF EXISTS product.product_variant CASCADE;
DROP TABLE IF EXISTS product.product CASCADE;
DROP SCHEMA IF EXISTS product CASCADE;
