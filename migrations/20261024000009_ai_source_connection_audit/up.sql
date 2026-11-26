-- ADR-035: actor-attributed source configuration changes; no credential values are recorded.
CREATE TABLE ai.source_connection_audit (
    tenant_id   text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    id          text        NOT NULL,
    source      text        NOT NULL CHECK (source = 'search_console'),
    account_id  text        NOT NULL CHECK (length(account_id) BETWEEN 1 AND 512),
    action      text        NOT NULL CHECK (action IN ('connected', 'updated', 'disabled', 'disconnected')),
    actor_id    text        NOT NULL CHECK (length(actor_id) BETWEEN 1 AND 128),
    created_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, id)
);

-- serves: GET /admin/ai/search-console/audit filtered by tenant/source/property newest first
CREATE INDEX idx_ai_source_connection_audit_recent
    ON ai.source_connection_audit (tenant_id, source, account_id, created_at DESC);

ALTER TABLE ai.source_connection_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.source_connection_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.source_connection_audit
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT ON ai.source_connection_audit TO stheno_app;
