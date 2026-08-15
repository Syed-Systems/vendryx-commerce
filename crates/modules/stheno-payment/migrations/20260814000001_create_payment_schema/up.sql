-- Phase: expand
-- Creates the payment module schema, its 8 core entities, and 3 Stheno safety tables.

CREATE SCHEMA IF NOT EXISTS payment;

-- 1. payment_collection
CREATE TABLE IF NOT EXISTS payment.payment_collection (
    id text PRIMARY KEY,
    currency_code text NOT NULL,
    amount numeric(20,6) NOT NULL,
    authorized_amount numeric(20,6) NOT NULL DEFAULT 0,
    captured_amount numeric(20,6) NOT NULL DEFAULT 0,
    refunded_amount numeric(20,6) NOT NULL DEFAULT 0,
    status text NOT NULL DEFAULT 'not_paid', -- 'not_paid', 'awaiting', 'authorized', 'partially_authorized', 'canceled'
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/payment-collections/:id
CREATE INDEX IF NOT EXISTS idx_payment_collection_status ON payment.payment_collection (status) WHERE deleted_at IS NULL;

-- 2. payment_provider
CREATE TABLE IF NOT EXISTS payment.payment_provider (
    id text PRIMARY KEY, -- e.g. 'pp_stripe_stripe', 'pp_system_default'
    is_enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- 3. payment_session
CREATE TABLE IF NOT EXISTS payment.payment_session (
    id text PRIMARY KEY,
    payment_collection_id text NOT NULL REFERENCES payment.payment_collection(id) ON DELETE CASCADE,
    provider_id text NOT NULL,
    amount numeric(20,6) NOT NULL,
    currency_code text NOT NULL,
    status text NOT NULL DEFAULT 'pending', -- 7 states: pending, pending_authorization, requires_more, authorized, captured, canceled, error
    data jsonb NULL,
    context jsonb NULL,
    authorized_at timestamptz NULL,
    status_observed_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /store/payment-collections/:id/payment-sessions
CREATE INDEX IF NOT EXISTS idx_payment_session_collection ON payment.payment_session (payment_collection_id) WHERE deleted_at IS NULL;

-- 4. payment (created ONLY upon successful authorization)
CREATE TABLE IF NOT EXISTS payment.payment (
    id text PRIMARY KEY,
    payment_collection_id text NOT NULL REFERENCES payment.payment_collection(id) ON DELETE CASCADE,
    payment_session_id text NULL REFERENCES payment.payment_session(id) ON DELETE SET NULL,
    provider_id text NOT NULL,
    currency_code text NOT NULL,
    amount numeric(20,6) NOT NULL,
    captured_at timestamptz NULL,
    canceled_at timestamptz NULL,
    data jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /admin/payments?payment_collection_id=
CREATE INDEX IF NOT EXISTS idx_payment_collection_id ON payment.payment (payment_collection_id) WHERE deleted_at IS NULL;

-- 5. refund_reason
CREATE TABLE IF NOT EXISTS payment.refund_reason (
    id text PRIMARY KEY,
    label text NOT NULL,
    description text NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- 6. capture (0 <= sum(captures) <= payment.amount)
CREATE TABLE IF NOT EXISTS payment.capture (
    id text PRIMARY KEY,
    payment_id text NOT NULL REFERENCES payment.payment(id) ON DELETE CASCADE,
    amount numeric(20,6) NOT NULL,
    created_by text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NULL
);

-- serves: GET /admin/payments/:id/captures
CREATE INDEX IF NOT EXISTS idx_capture_payment_id ON payment.capture (payment_id);

-- 7. refund (0 <= sum(refunds) <= sum(captures))
CREATE TABLE IF NOT EXISTS payment.refund (
    id text PRIMARY KEY,
    payment_id text NOT NULL REFERENCES payment.payment(id) ON DELETE CASCADE,
    amount numeric(20,6) NOT NULL,
    refund_reason_id text NULL REFERENCES payment.refund_reason(id) ON DELETE SET NULL,
    note text NULL,
    created_by text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    metadata jsonb NULL
);

-- serves: GET /admin/payments/:id/refunds
CREATE INDEX IF NOT EXISTS idx_refund_payment_id ON payment.refund (payment_id);

-- 8. account_holder
CREATE TABLE IF NOT EXISTS payment.account_holder (
    id text PRIMARY KEY,
    provider_id text NOT NULL,
    external_id text NOT NULL,
    email text NULL,
    data jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    metadata jsonb NULL
);

-- serves: GET /store/customers/me/account-holders?provider_id=
CREATE INDEX IF NOT EXISTS idx_account_holder_provider_external ON payment.account_holder (provider_id, external_id) WHERE deleted_at IS NULL;

-- ── 3 Stheno Safety Tables ───────────────────────────────────────────────────

-- 9. payment_provider_request (Outbound Idempotency Ledger, PAYMENTS §6)
CREATE TABLE IF NOT EXISTS payment.payment_provider_request (
    id text PRIMARY KEY,
    idempotency_key text NOT NULL UNIQUE,
    provider_id text NOT NULL,
    operation text NOT NULL, -- 'initiate', 'authorize', 'capture', 'refund', 'cancel'
    resource_type text NOT NULL,
    resource_id text NOT NULL,
    request_hash text NOT NULL, -- SHA-256 canonical hash
    state text NOT NULL DEFAULT 'in_flight', -- 'in_flight', 'succeeded', 'failed', 'unknown'
    response jsonb NULL,
    error jsonb NULL,
    attempts integer NOT NULL DEFAULT 1,
    first_sent_at timestamptz NOT NULL DEFAULT now(),
    settled_at timestamptz NULL
);

-- serves: Outbound idempotency check on provider mutation
CREATE INDEX IF NOT EXISTS idx_provider_req_key ON payment.payment_provider_request (idempotency_key);
-- serves: Ambiguous request sweep & reconciliation
CREATE INDEX IF NOT EXISTS idx_provider_req_state ON payment.payment_provider_request (state) WHERE state = 'unknown' OR state = 'in_flight';

-- 10. payment_webhook_event (Raw Webhook Event Store & Replay Guard, PAYMENTS §7)
CREATE TABLE IF NOT EXISTS payment.payment_webhook_event (
    id text PRIMARY KEY,
    provider_id text NOT NULL,
    provider_event_id text NOT NULL,
    event_type text NOT NULL,
    payload jsonb NOT NULL,
    signature_verified boolean NOT NULL DEFAULT false,
    provider_created_at timestamptz NOT NULL,
    state text NOT NULL DEFAULT 'received', -- 'received', 'processed', 'ignored', 'failed'
    processed_at timestamptz NULL,
    error text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT uq_provider_event UNIQUE (provider_id, provider_event_id)
);

-- serves: Webhook replay guard deduplication
CREATE INDEX IF NOT EXISTS idx_webhook_provider_event ON payment.payment_webhook_event (provider_id, provider_event_id);

-- 11. payment_reconciliation_run (Nightly Drift Detection Ledger, PAYMENTS §9)
CREATE TABLE IF NOT EXISTS payment.payment_reconciliation_run (
    id text PRIMARY KEY,
    provider_id text NOT NULL,
    window_start timestamptz NOT NULL,
    window_end timestamptz NOT NULL,
    checked_count integer NOT NULL DEFAULT 0,
    drift_count integer NOT NULL DEFAULT 0,
    drifts jsonb NULL,
    state text NOT NULL DEFAULT 'running', -- 'running', 'clean', 'drift_found', 'failed'
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz NULL
);

-- serves: GET /admin/payment-reconciliations?provider_id=
CREATE INDEX IF NOT EXISTS idx_reconciliation_provider ON payment.payment_reconciliation_run (provider_id, created_at DESC);
