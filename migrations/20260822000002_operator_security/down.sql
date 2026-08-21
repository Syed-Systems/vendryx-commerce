ALTER TABLE platform.operator
    DROP COLUMN IF EXISTS session_ttl_minutes,
    DROP COLUMN IF EXISTS ip_allowlist,
    DROP COLUMN IF EXISTS mfa_enabled,
    DROP COLUMN IF EXISTS mfa_secret,
    DROP COLUMN IF EXISTS role;
