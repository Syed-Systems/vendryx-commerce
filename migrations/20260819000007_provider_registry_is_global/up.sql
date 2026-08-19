-- `fulfillment.fulfillment_provider` is a build registry, not merchant data.
--
-- A row here means "this binary has this provider compiled in". It is seeded by
-- 20260815000001 with `('manual', 'Manual Fulfillment Provider')`, keyed by the code
-- identifier the provider returns from `identifier()`. Two tenants cannot legitimately
-- disagree about whether this build contains the manual provider — the same argument that
-- keeps `currency.currency` global.
--
-- Scoping it broke delivery outright. The migration's row was written before any tenant
-- existed, so it landed on `ten_default`; every real tenant's policy then hid it. The
-- provider list came back empty for all of them, and since `shipping_option.provider_id`
-- is a foreign key into this table, no tenant could create a shipping option at all. The
-- storefront's checkout reached the delivery step and had nothing to offer.
--
-- Per-tenant enablement is NOT lost, because it never lived here: it is
-- `links.link_fulfillment_provider_location` (and `links.link_region_payment_provider`
-- for payments), both of which stay tenant-scoped. This table answers "does this build
-- support it", the links answer "does this merchant use it".

ALTER TABLE fulfillment.fulfillment_provider DISABLE ROW LEVEL SECURITY;
ALTER TABLE fulfillment.fulfillment_provider NO FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON fulfillment.fulfillment_provider;

DROP INDEX IF EXISTS fulfillment.idx_fulfillment_provider_tenant;
ALTER TABLE fulfillment.fulfillment_provider DROP COLUMN IF EXISTS tenant_id;
