-- Phase: contract
-- Drops the cart module schema and all owned tables.

DROP TABLE IF EXISTS cart.credit_line;
DROP TABLE IF EXISTS cart.shipping_method_tax_line;
DROP TABLE IF EXISTS cart.shipping_method_adjustment;
DROP TABLE IF EXISTS cart.shipping_method;
DROP TABLE IF EXISTS cart.line_item_tax_line;
DROP TABLE IF EXISTS cart.line_item_adjustment;
DROP TABLE IF EXISTS cart.line_item;
DROP TABLE IF EXISTS cart.cart;
DROP TABLE IF EXISTS cart.address;

DROP SCHEMA IF EXISTS cart;
