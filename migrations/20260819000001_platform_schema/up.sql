-- ADR-012: one database, tenant_id on every row, isolation enforced by Postgres RLS.
--
-- This migration introduces the tenant registry itself. It deliberately does NOT add
-- tenant_id anywhere or enable RLS — those are the next two migrations, kept separate so
-- that turning isolation off in an incident is a fast, reversible step that does not drop
-- 157 columns.

CREATE SCHEMA IF NOT EXISTS platform;

-- The tenant registry. A tenant is one merchant; under ADR-009 this was a whole
-- deployment, which is exactly the cost ADR-012 removes.
CREATE TABLE IF NOT EXISTS platform.tenant (
    id          text PRIMARY KEY,
    name        text NOT NULL,
    status      text NOT NULL DEFAULT 'active',
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL,
    CONSTRAINT tenant_status_known CHECK (status IN ('active', 'suspended', 'closed'))
);

-- Host -> tenant. The storefront is unauthenticated, so the Host header is the only
-- tenant signal available on /store/*; a JWT can only ever cross-check it.
CREATE TABLE IF NOT EXISTS platform.tenant_domain (
    domain      text PRIMARY KEY,
    tenant_id   text NOT NULL REFERENCES platform.tenant(id) ON DELETE CASCADE,
    is_primary  boolean NOT NULL DEFAULT false,
    created_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_tenant_domain_tenant ON platform.tenant_domain (tenant_id);

-- What the tenant is entitled to. `tier` names a block in config/tiers.toml, which stays
-- the source of truth for what a tier *means*; this table only records the assignment,
-- plus per-tenant negotiated exceptions (ENTITLEMENTS.md 5b's additive env overrides,
-- persisted instead of set per process).
CREATE TABLE IF NOT EXISTS platform.subscription (
    tenant_id         text PRIMARY KEY REFERENCES platform.tenant(id) ON DELETE CASCADE,
    tier              text NOT NULL DEFAULT 'free',
    status            text NOT NULL DEFAULT 'active',
    trial_started_at  timestamptz NULL,
    trial_ends_at     timestamptz NULL,
    term_ends_at      timestamptz NULL,
    quota_overrides   jsonb NOT NULL DEFAULT '{}'::jsonb,
    window_overrides  jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz NULL,
    CONSTRAINT subscription_status_known CHECK (status IN ('active', 'grace', 'cancelled'))
);

-- The default tenant. Every row that exists today belongs to it, so the backfill in the
-- next migration has somewhere to point and existing deployments keep working unchanged.
INSERT INTO platform.tenant (id, name) VALUES ('ten_default', 'Default')
ON CONFLICT (id) DO NOTHING;

INSERT INTO platform.subscription (tenant_id, tier) VALUES ('ten_default', 'free')
ON CONFLICT (tenant_id) DO NOTHING;

-- entitlement_usage was keyed by resource alone, so every tenant would have contended for
-- three shared rows (ENTITLEMENTS.md 6.2 assumed one deployment per merchant).
ALTER TABLE entitlement_usage ADD COLUMN IF NOT EXISTS tenant_id text;
UPDATE entitlement_usage SET tenant_id = 'ten_default' WHERE tenant_id IS NULL;
ALTER TABLE entitlement_usage ALTER COLUMN tenant_id SET NOT NULL;
ALTER TABLE entitlement_usage ALTER COLUMN tenant_id
    SET DEFAULT current_setting('app.tenant_id', true);

ALTER TABLE entitlement_usage DROP CONSTRAINT IF EXISTS entitlement_usage_pkey;
ALTER TABLE entitlement_usage ADD PRIMARY KEY (tenant_id, resource);
