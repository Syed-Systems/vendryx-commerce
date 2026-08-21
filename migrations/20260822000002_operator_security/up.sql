-- P37-25/P37-08: operator role, MFA and network limits stop living in process memory.
--
-- P36 shipped `/platform/operators/{id}/role`, `/mfa/setup`, `/mfa/verify` and
-- `/network-limits` against a `static OnceLock<RwLock<HashMap<..>>>` in
-- crates/api/stheno-api-platform/src/security.rs. Restart the process and every operator's
-- role, MFA enrolment and IP allowlist is gone — which for an access-control surface means
-- the hardening silently stops applying, with no error anywhere.
--
-- These are columns on platform.operator rather than a table of their own: an operator's
-- role is an attribute of the operator in the same way `revoked_at` is, and a side table
-- would allow a profile row for an operator that does not exist — which is exactly the
-- fail-open the in-memory version had.
--
-- That fail-open is the second half of this migration and the more serious one. The
-- in-memory `get_security_profile` returned a default profile with `role: Owner` for any
-- id it had never seen, so an unknown operator id resolved to the MOST privileged role
-- rather than none. With the role on the operator row, an unknown operator has no row,
-- the lookup is NotFound, and the caller denies. The absent case fails closed by
-- construction instead of by remembering to check.

ALTER TABLE platform.operator
    -- DEFAULT 'owner' exists to backfill the operators that already exist, who predate
    -- roles and have been acting as owners. It is not a default for new credentials:
    -- issuing one names its role, and `check-control-plane-audit.sh` records who chose it.
    ADD COLUMN IF NOT EXISTS role text NOT NULL DEFAULT 'owner'
        CHECK (role IN ('owner', 'billing', 'support', 'engineer')),
    -- Enrolment and enforcement are separate: a secret is issued at setup, and
    -- `mfa_enabled` only becomes true once a code has been verified against it. Treating
    -- setup as enablement locks an operator out of their own control plane if they never
    -- finish scanning the code.
    ADD COLUMN IF NOT EXISTS mfa_secret text NULL,
    ADD COLUMN IF NOT EXISTS mfa_enabled boolean NOT NULL DEFAULT false,
    -- Empty means no restriction. A NOT NULL default of '[]' rather than a nullable
    -- column so "no allowlist" has exactly one representation.
    ADD COLUMN IF NOT EXISTS ip_allowlist jsonb NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS session_ttl_minutes integer NULL
        CHECK (session_ttl_minutes IS NULL OR session_ttl_minutes > 0);
