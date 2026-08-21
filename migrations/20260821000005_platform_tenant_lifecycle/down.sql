ALTER TABLE platform.tenant
    DROP COLUMN IF EXISTS status_changed_at,
    DROP COLUMN IF EXISTS status_reason;
