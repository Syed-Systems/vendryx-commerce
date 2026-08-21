-- Tenant lifecycle state needs an operator-visible reason now and a merchant-visible
-- suspension notice in P31-03.
ALTER TABLE platform.tenant
    ADD COLUMN IF NOT EXISTS status_reason text NULL,
    ADD COLUMN IF NOT EXISTS status_changed_at timestamptz NULL;
