ALTER TABLE entitlement_usage DROP CONSTRAINT IF EXISTS entitlement_usage_pkey;
ALTER TABLE entitlement_usage DROP COLUMN IF EXISTS tenant_id;
ALTER TABLE entitlement_usage ADD PRIMARY KEY (resource);

DROP SCHEMA IF EXISTS platform CASCADE;
