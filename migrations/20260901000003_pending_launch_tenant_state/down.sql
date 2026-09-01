UPDATE platform.tenant SET status = 'active' WHERE status = 'pending_launch';
ALTER TABLE platform.tenant DROP CONSTRAINT IF EXISTS tenant_status_known;
ALTER TABLE platform.tenant
    ADD CONSTRAINT tenant_status_known
    CHECK (status IN ('active', 'suspended', 'closed'));
