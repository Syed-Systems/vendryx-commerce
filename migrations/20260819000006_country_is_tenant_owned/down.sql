DROP POLICY IF EXISTS tenant_isolation ON region.country;
ALTER TABLE region.country NO FORCE ROW LEVEL SECURITY;
ALTER TABLE region.country DISABLE ROW LEVEL SECURITY;

DROP INDEX IF EXISTS region.idx_country_iso_2;
-- Restoring the global unique index can fail if two tenants now share a code. That is the
-- bug this migration fixed, so deduplicate before rolling back.
CREATE UNIQUE INDEX idx_country_iso_2 ON region.country (iso_2);

DROP INDEX IF EXISTS region.idx_country_tenant;
ALTER TABLE region.country DROP COLUMN IF EXISTS tenant_id;
