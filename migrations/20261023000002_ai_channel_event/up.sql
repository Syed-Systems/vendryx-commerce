-- ADR-032 phase 32.4: replay and out-of-order protection for inbound Meta message events.
-- Deliberately stores event identifiers and a hashed conversation key, never raw messages.

CREATE TABLE ai.channel_event (
    id                    text        PRIMARY KEY,
    tenant_id             text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    channel               text        NOT NULL CHECK (channel IN ('whatsapp', 'messenger', 'instagram')),
    external_account_id   text        NOT NULL,
    event_id              text        NOT NULL,
    conversation_key      text        NOT NULL,
    provider_created_at   timestamptz NOT NULL,
    state                 text        NOT NULL DEFAULT 'processing'
        CHECK (state IN ('processing', 'processed', 'failed', 'ignored')),
    processing_until      timestamptz,
    processed_at          timestamptz,
    recorded_at            timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT ai_channel_event_tenant_channel_account_event_uq
        UNIQUE (tenant_id, channel, external_account_id, event_id)
);

-- serves: serialized out-of-order checks for one tenant/channel/account/conversation
CREATE INDEX ai_channel_event_conversation_order_idx
    ON ai.channel_event (tenant_id, channel, external_account_id, conversation_key, provider_created_at DESC);

-- serves: reclaiming an event after handler outage or expired processing lease
CREATE INDEX ai_channel_event_retry_idx
    ON ai.channel_event (tenant_id, state, processing_until)
    WHERE state IN ('processing', 'failed');

ALTER TABLE ai.channel_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.channel_event FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON ai.channel_event
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

GRANT SELECT, INSERT, UPDATE, DELETE ON ai.channel_event TO stheno_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA ai
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO stheno_app;
