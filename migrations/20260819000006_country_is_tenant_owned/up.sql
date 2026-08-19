-- Corrects an exclusion made in 20260819000002: `region.country` is NOT reference data.
--
-- It was excluded alongside `currency.currency` on the argument that both are ISO tables
-- whose rows mean the same thing to every tenant. That argument holds for currencies —
-- `currency.currency` has a code, a symbol and a name, and nothing else. It does not hold
-- here, because `region.country` carries **region_id**, a foreign key into a tenant-owned
-- table. A row saying "us belongs to region reg_01M0D4…" is not a fact about ISO 3166; it
-- is a fact about one merchant's setup.
--
-- The failure this produced was total and silent. `idx_country_iso_2` was globally unique,
-- so of three tenants seeding the same countries only the FIRST got rows; the other two
-- got a unique violation on a row they could not read, swallowed by the seed's `let _ =`.
-- Their regions then returned `countries: []`, the storefront's middleware built an empty
-- region map, and every page answered 500 — no product page, no cart, no checkout, for
-- every tenant but one.
--
-- It was also a read leak in the opposite direction to the usual one: with no RLS on this
-- table, any tenant could enumerate every other tenant's region ids.

-- Add nullable, backfill, then constrain — the same order 20260819000002 uses, because
-- the DEFAULT reads `app.tenant_id`, which is unset while migrations run. Declaring the
-- column NOT NULL DEFAULT in one statement makes every existing row NULL and fails.
ALTER TABLE region.country ADD COLUMN IF NOT EXISTS tenant_id text;

-- Existing rows belong to whichever tenant's regions they already point at. Rows with no
-- region (none today, but the FK is ON DELETE SET NULL so they can appear) go to the
-- default tenant rather than being deleted.
UPDATE region.country c
   SET tenant_id = COALESCE(
       (SELECT r.tenant_id FROM region.region r WHERE r.id = c.region_id),
       'ten_default'
   )
 WHERE c.tenant_id IS NULL;

ALTER TABLE region.country ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE region.country
    ALTER COLUMN tenant_id SET DEFAULT current_setting('app.tenant_id', true);

CREATE INDEX IF NOT EXISTS idx_country_tenant ON region.country (tenant_id);

-- The whole point: two tenants may both configure "us".
DROP INDEX IF EXISTS region.idx_country_iso_2;
CREATE UNIQUE INDEX idx_country_iso_2 ON region.country (tenant_id, iso_2);

ALTER TABLE region.country ENABLE ROW LEVEL SECURITY;
-- FORCE, because the app owns this table and owners are otherwise exempt.
ALTER TABLE region.country FORCE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation ON region.country;
CREATE POLICY tenant_isolation ON region.country
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
