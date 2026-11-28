-- ADR-035: bind one-time OAuth callbacks to the initiating tenant, property, and actor.
CREATE TABLE ai.search_console_oauth_state (
    tenant_id   text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    nonce       text        NOT NULL CHECK (length(nonce) = 26),
    property    text        NOT NULL CHECK (length(property) BETWEEN 1 AND 512),
    actor_id    text        NOT NULL CHECK (length(actor_id) BETWEEN 1 AND 128),
    expires_at  timestamptz NOT NULL DEFAULT (now() + interval '10 minutes'),
    PRIMARY KEY (tenant_id, nonce)
);

-- serves: prune expired states when an owner starts a new Search Console connection
CREATE INDEX idx_ai_search_console_oauth_state_expiry
    ON ai.search_console_oauth_state (expires_at);

ALTER TABLE ai.search_console_oauth_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.search_console_oauth_state FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.search_console_oauth_state
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT, DELETE ON ai.search_console_oauth_state TO stheno_app;
