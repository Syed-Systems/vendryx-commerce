-- phase: contract
ALTER TABLE platform.subscription DROP COLUMN IF EXISTS feature_overrides;
DROP TABLE IF EXISTS platform.tier_definition;
