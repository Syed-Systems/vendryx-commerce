-- ADR-032 phase 32.5: encrypted per-tenant AI provider and Meta channel credentials.
-- Ciphertext is tenant/key-purpose bound by AES-GCM associated data in the module service.

CREATE TABLE ai.configuration (
    id             text        PRIMARY KEY,
    tenant_id      text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    kind           text        NOT NULL CHECK (kind IN ('llm', 'whatsapp', 'messenger', 'instagram')),
    account_id     text        NOT NULL,
    webhook_id     text,
    ciphertext     bytea       NOT NULL,
    nonce          bytea       NOT NULL CHECK (octet_length(nonce) = 12),
    has_credentials boolean    NOT NULL DEFAULT false,
    version        integer     NOT NULL DEFAULT 1 CHECK (version > 0),
    enabled        boolean     NOT NULL DEFAULT false,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, kind, account_id)
);

-- serves: GET/PUT /admin/ai configuration by tenant and kind/account
CREATE UNIQUE INDEX ai_configuration_webhook_id_uq
    ON ai.configuration (webhook_id)
    WHERE webhook_id IS NOT NULL;

ALTER TABLE ai.configuration ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.configuration FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON ai.configuration
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

GRANT SELECT, INSERT, UPDATE, DELETE ON ai.configuration TO stheno_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA ai
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO stheno_app;
