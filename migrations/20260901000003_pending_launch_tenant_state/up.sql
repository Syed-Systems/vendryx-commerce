-- A launch-owned tenant must not become customer-visible merely because its database
-- registration succeeded. It remains pending until the audited activation handoff.
ALTER TABLE platform.tenant DROP CONSTRAINT IF EXISTS tenant_status_known;
ALTER TABLE platform.tenant
    ADD CONSTRAINT tenant_status_known
    CHECK (status IN ('pending_launch', 'active', 'suspended', 'closed'));
