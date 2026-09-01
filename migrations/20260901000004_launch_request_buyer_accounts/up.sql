-- The buyer-account archetype the merchant picks on the store request (PRODUCT.md 2).
--
-- Not a tier lever: every tier can run either archetype, so this is not derivable from
-- `tier` and does not belong in config/tiers.toml. Provisioning reads it and writes the
-- per-merchant `buyer_accounts` entitlement override.
--
-- Defaults false, which is the archetype the product has always described: buyers order
-- without an account. Every request that already exists keeps that behaviour.
ALTER TABLE platform.launch_request
    ADD COLUMN IF NOT EXISTS buyer_accounts boolean NOT NULL DEFAULT false;
