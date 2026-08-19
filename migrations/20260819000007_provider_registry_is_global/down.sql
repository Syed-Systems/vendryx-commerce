ALTER TABLE fulfillment.fulfillment_provider
    ADD COLUMN IF NOT EXISTS tenant_id text;
UPDATE fulfillment.fulfillment_provider SET tenant_id = 'ten_default' WHERE tenant_id IS NULL;
ALTER TABLE fulfillment.fulfillment_provider ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE fulfillment.fulfillment_provider
    ALTER COLUMN tenant_id SET DEFAULT current_setting('app.tenant_id', true);
CREATE INDEX IF NOT EXISTS idx_fulfillment_provider_tenant
    ON fulfillment.fulfillment_provider (tenant_id);

ALTER TABLE fulfillment.fulfillment_provider ENABLE ROW LEVEL SECURITY;
ALTER TABLE fulfillment.fulfillment_provider FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON fulfillment.fulfillment_provider
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
