-- The account-to-tenant handoff is durable. A retry sees this row and resumes the same
-- deployment rather than giving a merchant a second tenant or a second trial.
CREATE TABLE IF NOT EXISTS platform.merchant_tenant_owner (
    tenant_id   text PRIMARY KEY REFERENCES platform.tenant(id) ON DELETE CASCADE,
    merchant_id text NOT NULL REFERENCES platform.merchant_account(id) ON DELETE RESTRICT,
    bound_at    timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_merchant_tenant_owner_merchant
    ON platform.merchant_tenant_owner (merchant_id)
    WHERE deleted_at IS NULL;
