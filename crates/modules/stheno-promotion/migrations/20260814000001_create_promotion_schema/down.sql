-- Phase: contract
-- Drops the promotion module schema and all owned tables.

DROP TABLE IF EXISTS promotion.promotion_rule_value;
DROP TABLE IF EXISTS promotion.promotion_rule;
DROP TABLE IF EXISTS promotion.application_method;
DROP TABLE IF EXISTS promotion.promotion;
DROP TABLE IF EXISTS promotion.campaign_budget_usage;
DROP TABLE IF EXISTS promotion.campaign_budget;
DROP TABLE IF EXISTS promotion.campaign;

DROP SCHEMA IF EXISTS promotion;
