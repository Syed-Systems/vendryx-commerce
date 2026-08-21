-- P37-02/03/04/08: the control plane stops keeping its state in process memory.
--
-- P32, P34 and P35 shipped nine `static OnceLock<RwLock<HashMap<..>>>` registries inside
-- crates/api: the gateway kill switch, impersonation sessions, the emergency read-only
-- lock, tenant theme assignments, global feature flags, deployment defaults, messaging
-- config and announcements. Every one of them is lost on restart and was never shared
-- between nodes, so an operator who kills a gateway on node A has not killed it on node B,
-- and a merchant put read-only during an incident is writable again the moment the process
-- recycles. `scripts/check-no-in-memory-stores.sh` did not catch it because it only ever
-- scanned crates/modules; P37-01 widened it and records these nine as a shrinking ledger.
--
-- Everything here is global rather than tenant-scoped, for the same reason as
-- platform.operator and platform.tier_definition: `platform` is outside the RLS schema
-- list (20260819000003), and all of it belongs to the deployment rather than to one
-- merchant. Which tenant a row is *about* is a column, never the row's owner.

-- ── Global flags: gateway kill switches and deployment feature flags ─────────────────
--
-- One table, two kinds, because they are the same thing: a named boolean the deployment
-- owns, carrying who set it and why. Splitting them would duplicate the reason/actor
-- columns and the "who turned this off" question would need asking twice.
--
-- ABSENT ROW MEANS ENABLED, and that is deliberate for both kinds. A kill switch is an
-- override, not an authorisation: if a missing row meant "killed", every gateway would be
-- dead on a fresh deployment until someone inserted rows for each. Rule 3 (fails closed)
-- governs credentials and access, not overrides — the closed state for an override is
-- "not overriding". Killing something is an explicit row with is_enabled = false.
CREATE TABLE IF NOT EXISTS platform.global_flag (
    -- A provider id (`pp_bkash`) for a gateway, a flag name (`checkout_v2`) for a feature.
    key                 text PRIMARY KEY,
    kind                text NOT NULL CHECK (kind IN ('gateway', 'feature')),
    description         text NOT NULL DEFAULT '',
    is_enabled          boolean NOT NULL DEFAULT true,
    -- Percentage rollout for `feature` rows. A `gateway` row is on or off; a partial
    -- kill switch would mean some merchants take payments and some do not, with no way
    -- to say which, so gateways are constrained to 100 below.
    rollout_percentage  smallint NOT NULL DEFAULT 100
                        CHECK (rollout_percentage BETWEEN 0 AND 100),
    -- Why it was turned off. An incident switch with no reason is unreadable a week later.
    reason              text NULL,
    -- The operator id, or `op_bootstrap`. Denormalised from platform.operator so the row
    -- still reads after a credential is revoked, same rule as platform.operator_audit.
    updated_by          text NULL,
    updated_at          timestamptz NOT NULL DEFAULT now(),
    -- `PgCrudRepository::retrieve` predicates on `deleted_at IS NULL` unconditionally.
    deleted_at          timestamptz NULL,
    CONSTRAINT global_flag_gateway_is_all_or_nothing
        CHECK (kind <> 'gateway' OR rollout_percentage = 100)
);

CREATE INDEX IF NOT EXISTS idx_global_flag_disabled
    ON platform.global_flag (kind) WHERE is_enabled = false AND deleted_at IS NULL;

-- ── Impersonation sessions (ADR-018) ────────────────────────────────────────────────
--
-- The token HASH, never the token. A support session token sitting in a table any
-- operator can read is a credential leak with an audit trail attached — the same reason
-- platform.operator stores token_hash and a redacted display form.
--
-- Expiry is enforced by reading `expires_at`, not by a timer: an in-process timer dies
-- with the process it runs in, which is exactly the class of bug this migration exists to
-- remove. A session is live when revoked_at IS NULL AND expires_at > now().
CREATE TABLE IF NOT EXISTS platform.impersonation_session (
    id            text PRIMARY KEY,
    token_hash    text NOT NULL UNIQUE,
    -- `imp_...abcd`. Enough to correlate a session in the UI, not enough to use.
    redacted_token text NOT NULL,
    tenant_id     text NOT NULL,
    operator_id   text NOT NULL,
    -- Required by ADR-018. A support session with no stated reason is not consented to.
    reason        text NOT NULL,
    -- What the session may do, as granted at issue time. Stored rather than recomputed so
    -- narrowing the scope rules later cannot retroactively widen a session already issued.
    scopes        jsonb NOT NULL DEFAULT '[]'::jsonb,
    created_at    timestamptz NOT NULL DEFAULT now(),
    expires_at    timestamptz NOT NULL,
    -- Ending a session early is a timestamp, not a delete: the audit rows naming this
    -- session must still resolve.
    revoked_at    timestamptz NULL,
    deleted_at    timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_impersonation_live
    ON platform.impersonation_session (token_hash) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_impersonation_by_tenant
    ON platform.impersonation_session (tenant_id, created_at DESC);

-- ── Emergency read-only lock ────────────────────────────────────────────────────────
--
-- A column on the tenant, not a side table: this is tenant state in exactly the way
-- `status` is, and a merchant cannot be read-only without existing. A side table would
-- also let the two disagree — closed and read-only, suspended and writable.
ALTER TABLE platform.tenant
    ADD COLUMN IF NOT EXISTS is_read_only boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS read_only_reason text NULL,
    ADD COLUMN IF NOT EXISTS read_only_since timestamptz NULL;

-- ── Tenant theme assignment (D7) ────────────────────────────────────────────────────
--
-- One row per tenant, so a merchant cannot be on two themes. The theme catalogue itself
-- stays in code — a theme is a package that ships with the build (P27-08), not a row —
-- and this table records only which of them a merchant is pinned to.
CREATE TABLE IF NOT EXISTS platform.tenant_theme (
    tenant_id    text PRIMARY KEY,
    theme_id     text NOT NULL,
    -- Pinned deliberately. A theme that floats to latest can change a merchant's
    -- storefront without anyone deploying, which is the opposite of P27-03's intent.
    version      text NOT NULL,
    is_custom    boolean NOT NULL DEFAULT false,
    assigned_by  text NULL,
    assigned_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at   timestamptz NULL
);

-- ── Deployment configuration (D8) ───────────────────────────────────────────────────
--
-- Exactly one row, enforced by the CHECK rather than by convention: two rows of
-- deployment defaults means the answer to "what is the default currency" depends on
-- which one you read first.
--
-- `default_tax_rate` is numeric, not double precision. ADR-006 keeps floats away from
-- anything that multiplies money, and a tax rate multiplies every order total in the
-- deployment. The in-memory DTO it replaces used f64; the DTO changes with it.
CREATE TABLE IF NOT EXISTS platform.deployment_config (
    id                          text PRIMARY KEY DEFAULT 'singleton'
                                CHECK (id = 'singleton'),
    default_region              text NOT NULL,
    default_currency            text NOT NULL,
    default_tax_rate            numeric(6, 4) NOT NULL DEFAULT 0,
    support_email               text NOT NULL,
    email_provider              text NOT NULL DEFAULT '',
    sms_provider                text NOT NULL DEFAULT '',
    from_email                  text NOT NULL DEFAULT '',
    order_confirmation_template text NOT NULL DEFAULT '',
    updated_by                  text NULL,
    updated_at                  timestamptz NOT NULL DEFAULT now(),
    deleted_at                  timestamptz NULL
);

-- ── Broadcast announcements (D8) ────────────────────────────────────────────────────
--
-- Expiry is a column and never a cleanup job: a merchant-visible banner that outlives its
-- incident is worse than one that never appeared, and a delete would destroy the record
-- of what merchants were told during an outage.
CREATE TABLE IF NOT EXISTS platform.announcement (
    id          text PRIMARY KEY,
    title       text NOT NULL,
    message     text NOT NULL,
    severity    text NOT NULL CHECK (severity IN ('info', 'warning', 'critical')),
    created_by  text NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL,
    deleted_at  timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_announcement_live
    ON platform.announcement (expires_at DESC) WHERE deleted_at IS NULL;
