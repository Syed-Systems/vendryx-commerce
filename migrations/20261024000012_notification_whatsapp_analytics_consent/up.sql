-- ADR-035: keep anonymous analytics consent separate from transactional WhatsApp opt-in.
CREATE TABLE notification.whatsapp_analytics_consent (
    tenant_id       text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    recipient_hash  text        NOT NULL CHECK (recipient_hash ~ '^[0-9a-f]{64}$'),
    customer_id     text,
    source_event_id text        NOT NULL CHECK (length(source_event_id) BETWEEN 1 AND 128),
    consented       boolean     NOT NULL DEFAULT true,
    consent_source  text        NOT NULL CHECK (consent_source IN ('checkout', 'customer_settings')),
    consented_at    timestamptz NOT NULL DEFAULT now(),
    revoked_at      timestamptz,
    PRIMARY KEY (tenant_id, recipient_hash),
    CHECK ((consented AND revoked_at IS NULL) OR (NOT consented AND revoked_at IS NOT NULL))
);

-- serves: revoke anonymous WhatsApp analytics consent for an authenticated customer
CREATE INDEX idx_whatsapp_analytics_consent_customer_active
    ON notification.whatsapp_analytics_consent (tenant_id, customer_id)
    WHERE customer_id IS NOT NULL AND consented = true;

ALTER TABLE notification.whatsapp_analytics_consent ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.whatsapp_analytics_consent FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON notification.whatsapp_analytics_consent
    USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
    WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));
GRANT SELECT, INSERT, UPDATE, DELETE ON notification.whatsapp_analytics_consent TO stheno_app;

COMMENT ON TABLE notification.whatsapp_analytics_consent IS
    'Explicit opt-in for anonymous tenant WhatsApp event aggregates; phone numbers are never stored.';
