-- Merchant accounts and launch requests live in the platform schema because they exist before a
-- tenant exists. They must never be created inside an individual store deployment.

CREATE TABLE IF NOT EXISTS platform.merchant_account (
    id                  text PRIMARY KEY,
    email               text NOT NULL,
    email_verified_at   timestamptz NULL,
    signup_draft        jsonb NOT NULL DEFAULT '{}'::jsonb,
    selected_tier       text NOT NULL DEFAULT 'free',
    created_at          timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    -- Never set: these entities hard-delete (see `is_deleted` in models.rs). The column
    -- exists because PgCrudRepository::retrieve unconditionally emits
    -- `AND t.deleted_at IS NULL`, so a table without it cannot be read at all — the
    -- symptom is `column t.deleted_at does not exist` from an otherwise valid query.
    deleted_at          timestamptz NULL,
    CONSTRAINT merchant_account_tier_known CHECK (selected_tier IN ('free', 'pro'))
);

CREATE TABLE IF NOT EXISTS platform.launch_request (
    id                  text PRIMARY KEY,
    merchant_id         text NOT NULL REFERENCES platform.merchant_account(id) ON DELETE RESTRICT,
    store_name          text NOT NULL,
    subdomain           text NOT NULL UNIQUE,
    tier                text NOT NULL,
    payment_status      text NOT NULL DEFAULT 'not_required',
    launch_status       text NOT NULL DEFAULT 'submitted',
    logo_url            text NULL,
    accent_color        text NOT NULL,
    icon_choice         text NOT NULL,
    bespoke_brief       text NULL,
    bespoke_assets      jsonb NOT NULL DEFAULT '[]'::jsonb,
    validation_findings jsonb NOT NULL DEFAULT '[]'::jsonb,
    assigned_developer  text NULL,
    tenant_id           text NULL REFERENCES platform.tenant(id) ON DELETE SET NULL,
    submitted_at        timestamptz NOT NULL DEFAULT now(),
    updated_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz NULL,
    CONSTRAINT launch_request_tier_known CHECK (tier IN ('free', 'pro')),
    CONSTRAINT launch_request_payment_known CHECK (payment_status IN ('not_required', 'pending', 'confirmed', 'failed')),
    CONSTRAINT launch_request_status_known CHECK (launch_status IN ('submitted', 'in_review', 'provisioning', 'online', 'needs_changes', 'rejected')),
    CONSTRAINT launch_request_pro_payment CHECK (tier <> 'pro' OR payment_status <> 'not_required')
);

CREATE INDEX IF NOT EXISTS idx_launch_request_queue
    ON platform.launch_request (launch_status, submitted_at DESC);
CREATE INDEX IF NOT EXISTS idx_launch_request_merchant
    ON platform.launch_request (merchant_id, submitted_at DESC);

CREATE TABLE IF NOT EXISTS platform.launch_request_event (
    id                  text PRIMARY KEY,
    launch_request_id   text NOT NULL REFERENCES platform.launch_request(id) ON DELETE CASCADE,
    actor_id            text NOT NULL,
    actor_name          text NOT NULL,
    action              text NOT NULL,
    note                text NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    deleted_at          timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_launch_request_event_timeline
    ON platform.launch_request_event (launch_request_id, created_at DESC);
