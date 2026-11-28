-- ADR-032 phase 32.5: short-lived encrypted multichannel conversation context.
-- Buyer PII and chat contents never enter plaintext columns or the knowledge corpus.
CREATE TABLE ai.channel_conversation (
    tenant_id           text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    channel             text        NOT NULL CHECK (channel IN ('whatsapp', 'messenger', 'instagram')),
    external_account_id text        NOT NULL,
    conversation_key    text        NOT NULL,
    ciphertext          bytea       NOT NULL,
    nonce               bytea       NOT NULL CHECK (octet_length(nonce) = 12),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    expires_at          timestamptz NOT NULL,
    PRIMARY KEY (tenant_id, channel, external_account_id, conversation_key)
);

-- serves: tenant-scoped conversation history load, upsert, and expiry cleanup
CREATE INDEX ai_channel_conversation_expiry_idx
    ON ai.channel_conversation (tenant_id, expires_at);

ALTER TABLE ai.channel_conversation ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.channel_conversation FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.channel_conversation
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT, UPDATE, DELETE ON ai.channel_conversation TO stheno_app;
