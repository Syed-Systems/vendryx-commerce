-- Phase: expand
-- P21-C1: tenant-owned encrypted provider credentials.
--
-- This table stores only encrypted credential material. Plaintext values are accepted at
-- the API boundary, encrypted in application code, and never written to Postgres.
CREATE TABLE IF NOT EXISTS payment.payment_provider_credential (
    id                 text PRIMARY KEY,
    tenant_id          text NOT NULL DEFAULT current_setting('app.tenant_id', true),
    provider_id        text NOT NULL,
    ciphertext         bytea NOT NULL,
    nonce              bytea NOT NULL,
    credential_version integer NOT NULL DEFAULT 1,
    webhook_slug       text NOT NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz
);

-- serves: GET/PUT /admin/payment-providers/:provider_id/credentials
CREATE UNIQUE INDEX IF NOT EXISTS payment_provider_credential_tenant_provider_uq
    ON payment.payment_provider_credential (tenant_id, provider_id)
    WHERE deleted_at IS NULL;

-- serves: POST /hooks/payment/:webhook_slug tenant/provider resolution before signature verification
CREATE UNIQUE INDEX IF NOT EXISTS payment_provider_credential_webhook_slug_uq
    ON payment.payment_provider_credential (webhook_slug)
    WHERE deleted_at IS NULL;

-- serves: provider credential cache invalidation by tenant/provider/version
CREATE INDEX IF NOT EXISTS payment_provider_credential_cache_key_idx
    ON payment.payment_provider_credential (tenant_id, provider_id, credential_version)
    WHERE deleted_at IS NULL;

ALTER TABLE payment.payment_provider_credential ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment.payment_provider_credential FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON payment.payment_provider_credential
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
