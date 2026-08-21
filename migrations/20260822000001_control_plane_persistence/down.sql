DROP TABLE IF EXISTS platform.announcement;
DROP TABLE IF EXISTS platform.deployment_config;
DROP TABLE IF EXISTS platform.tenant_theme;
DROP TABLE IF EXISTS platform.impersonation_session;
DROP TABLE IF EXISTS platform.global_flag;

ALTER TABLE platform.tenant
    DROP COLUMN IF EXISTS read_only_since,
    DROP COLUMN IF EXISTS read_only_reason,
    DROP COLUMN IF EXISTS is_read_only;
