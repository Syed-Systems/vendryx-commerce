CREATE TABLE notification.notification_outbox (
    id text PRIMARY KEY,
    tenant_id text NOT NULL,
    notification_id text NOT NULL UNIQUE REFERENCES notification.notification(id) ON DELETE CASCADE,
    idempotency_key text NOT NULL UNIQUE,
    provider_id text NOT NULL,
    "to" text NOT NULL,
    channel text NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    subject text NULL,
    body text NOT NULL,
    data jsonb NULL,
    trigger_type text NULL,
    resource_id text NULL,
    resource_type text NULL,
    status text NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'sending', 'sent', 'dead')),
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0 AND attempts <= 12),
    available_at timestamptz NOT NULL DEFAULT now(),
    locked_until timestamptz NULL,
    last_error text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    sent_at timestamptz NULL
);

-- serves: notification delivery worker claim of due/recoverable messages
CREATE INDEX idx_notification_outbox_due
    ON notification.notification_outbox (available_at, created_at)
    WHERE status IN ('pending', 'sending');

COMMENT ON TABLE notification.notification_outbox IS
    'Durable email/WhatsApp delivery intents. Retain payloads only for the configured notification retention window.';

CREATE TABLE notification.whatsapp_consent (
    tenant_id text NOT NULL,
    recipient_e164 text NOT NULL,
    customer_id text NULL,
    consented boolean NOT NULL DEFAULT true,
    consent_source text NOT NULL CHECK (consent_source IN ('checkout', 'chat', 'customer_settings')),
    consented_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, recipient_e164),
    CHECK (recipient_e164 ~ '^\+[1-9][0-9]{7,14}$'),
    CHECK ((consented AND revoked_at IS NULL) OR (NOT consented AND revoked_at IS NOT NULL))
);

-- serves: DELETE /store/customers/me/whatsapp-consent
CREATE INDEX idx_whatsapp_consent_customer_active
    ON notification.whatsapp_consent (tenant_id, customer_id)
    WHERE customer_id IS NOT NULL AND consented = true;

ALTER TABLE notification.whatsapp_consent ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.whatsapp_consent FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON notification.whatsapp_consent
    USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
    WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));

COMMENT ON TABLE notification.whatsapp_consent IS
    'Tenant-scoped affirmative WhatsApp notification consent and revocation ledger.';

CREATE TABLE notification.daily_notification_usage (
    tenant_id text NOT NULL,
    channel text NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    usage_date date NOT NULL,
    reserved_count integer NOT NULL DEFAULT 0 CHECK (reserved_count >= 0),
    sent_count integer NOT NULL DEFAULT 0 CHECK (sent_count >= 0),
    PRIMARY KEY (tenant_id, channel, usage_date)
);

ALTER TABLE notification.daily_notification_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.daily_notification_usage FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON notification.daily_notification_usage
    USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
    WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));

CREATE TABLE notification.daily_notification_reservation (
    outbox_id text NOT NULL,
    tenant_id text NOT NULL,
    channel text NOT NULL CHECK (channel IN ('email', 'whatsapp')),
    usage_date date NOT NULL,
    status text NOT NULL DEFAULT 'reserved' CHECK (status IN ('reserved', 'sent')),
    created_at timestamptz NOT NULL DEFAULT now(),
    sent_at timestamptz NULL,
    PRIMARY KEY (outbox_id, usage_date),
    FOREIGN KEY (tenant_id, channel, usage_date)
        REFERENCES notification.daily_notification_usage (tenant_id, channel, usage_date)
        ON DELETE CASCADE
);

ALTER TABLE notification.daily_notification_reservation ENABLE ROW LEVEL SECURITY;
ALTER TABLE notification.daily_notification_reservation FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON notification.daily_notification_reservation
    USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
    WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)));

COMMENT ON TABLE notification.daily_notification_usage IS
    'UTC-day outbound notification quota reservations and accepted delivery counts by tenant and channel.';
