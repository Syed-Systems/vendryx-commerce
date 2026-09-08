-- P21-05: Raw webhook event store & replay guard for couriers/fulfillment providers.
-- Tenant-owned, RLS + FORCE, unique on (tenant_id, provider_id, provider_event_id).
CREATE TABLE IF NOT EXISTS fulfillment.fulfillment_webhook_event (
    id                  text PRIMARY KEY,
    tenant_id           text NOT NULL DEFAULT current_setting('app.tenant_id', true),
    provider_id         text NOT NULL,
    provider_event_id   text NOT NULL,
    event_type          text NOT NULL,
    payload             jsonb NOT NULL,
    signature_verified  boolean NOT NULL DEFAULT false,
    provider_created_at timestamptz NOT NULL,
    state               text NOT NULL DEFAULT 'received', -- 'received', 'processed', 'ignored', 'failed'
    processed_at        timestamptz,
    error               text,
    created_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz
);

-- serves: Fulfillment webhook replay guard deduplication
CREATE UNIQUE INDEX IF NOT EXISTS fulfillment_webhook_event_provider_event_uq
    ON fulfillment.fulfillment_webhook_event (tenant_id, provider_id, provider_event_id)
    WHERE deleted_at IS NULL;

-- serves: Event processing sweep and auditing
CREATE INDEX IF NOT EXISTS fulfillment_webhook_event_state_idx
    ON fulfillment.fulfillment_webhook_event (state, created_at)
    WHERE deleted_at IS NULL;

ALTER TABLE fulfillment.fulfillment_webhook_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE fulfillment.fulfillment_webhook_event FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON fulfillment.fulfillment_webhook_event
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
