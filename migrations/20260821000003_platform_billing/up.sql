-- Subscription billing: what a merchant owes us, and whether they paid it (ADR-017).
--
-- The business had a plan editor, per-merchant negotiated exceptions, and no way to charge
-- anyone for any of it. `platform.tenant.status` shipped with a `suspended` value that no
-- code path wrote, because nothing could decide a merchant had not paid.
--
-- Global, in the `platform` schema, for the reason ADR-017 decision 1 gives: a merchant's
-- subscription invoice is *our* revenue record, not one of that merchant's commerce
-- records. Putting it in an RLS-scoped table would make it readable — and writable —
-- through the merchant's own surface.
--
-- Money is `numeric(20,6)` plus a currency column, per ADR-006. Not minor units: proration
-- produces sub-minor intermediates, which is the exact case that ADR exists for.

-- One period, one merchant, one bill.
CREATE TABLE IF NOT EXISTS platform.invoice (
    id             text PRIMARY KEY,
    tenant_id      text NOT NULL REFERENCES platform.tenant(id) ON DELETE CASCADE,
    -- The plan the merchant was on when the period was billed. Denormalised because the
    -- invoice must still read correctly after they move plans.
    tier           text NOT NULL,
    period_start   timestamptz NOT NULL,
    period_end     timestamptz NOT NULL,
    currency       text NOT NULL,
    -- Sum of the non-tax lines.
    subtotal       numeric(20,6) NOT NULL DEFAULT 0,
    tax_total      numeric(20,6) NOT NULL DEFAULT 0,
    -- Credits and discounts, stored positive and subtracted.
    credit_total   numeric(20,6) NOT NULL DEFAULT 0,
    total          numeric(20,6) NOT NULL DEFAULT 0,
    status         text NOT NULL DEFAULT 'open',
    issued_at      timestamptz NOT NULL DEFAULT now(),
    paid_at        timestamptz NULL,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz NULL,

    CONSTRAINT invoice_status_known
        CHECK (status IN ('open', 'paid', 'void', 'uncollectible')),

    -- A total can be zero (a promotional plan) but never negative: an invoice that pays
    -- the merchant is a credit note, which this table does not model.
    CONSTRAINT invoice_total_not_negative CHECK (total >= 0),

    -- THE idempotency guarantee for the billing run (P30-04).
    --
    -- Enforced by the database rather than by a check-then-insert in Rust, because the
    -- billing run is a scheduled job that may execute on two nodes at once. A read
    -- followed by a write loses that race and bills the merchant twice; a unique index
    -- cannot.
    CONSTRAINT invoice_one_per_period UNIQUE (tenant_id, period_start)
);

CREATE INDEX IF NOT EXISTS idx_invoice_tenant ON platform.invoice (tenant_id, period_start DESC);
CREATE INDEX IF NOT EXISTS idx_invoice_open ON platform.invoice (status, issued_at) WHERE status = 'open';

-- What the invoice is made of. Kept as rows rather than folded into the totals so that a
-- disputed bill can be explained line by line rather than defended as an arithmetic claim.
CREATE TABLE IF NOT EXISTS platform.invoice_line (
    id          text PRIMARY KEY,
    invoice_id  text NOT NULL REFERENCES platform.invoice(id) ON DELETE CASCADE,
    -- subscription | proration_credit | proration_charge | tax | credit | discount
    kind        text NOT NULL,
    description text NOT NULL,
    -- Signed. A credit line is negative, so the lines sum to the total and a reader does
    -- not have to know which kinds are subtracted.
    amount      numeric(20,6) NOT NULL,
    created_at  timestamptz NOT NULL DEFAULT now(),
    deleted_at  timestamptz NULL,

    CONSTRAINT invoice_line_kind_known
        CHECK (kind IN ('subscription', 'proration_credit', 'proration_charge', 'tax', 'credit', 'discount'))
);

CREATE INDEX IF NOT EXISTS idx_invoice_line_invoice ON platform.invoice_line (invoice_id);

-- The outbound ledger for collection.
--
-- Same shape and same states as `payment.payment_provider_request`, deliberately not the
-- same table — ADR-017 decision 2. A row is written BEFORE the provider is called, so an
-- ambiguous timeout leaves evidence that a charge may have happened. Without that, a retry
-- after a timeout is a second charge to a merchant who is already reluctant to pay us.
CREATE TABLE IF NOT EXISTS platform.billing_attempt (
    id                 text PRIMARY KEY,
    invoice_id         text NOT NULL REFERENCES platform.invoice(id) ON DELETE CASCADE,
    tenant_id          text NOT NULL,
    provider_id        text NOT NULL,
    -- Derived from the invoice and the attempt number, so a retry of the same attempt
    -- converges and a genuinely new attempt does not collide with it.
    idempotency_key    text NOT NULL UNIQUE,
    -- SHA-256 of the request payload. A key reused with a different payload is a bug in
    -- the caller and is refused rather than serving the first response for the second call.
    request_hash       text NOT NULL,
    state              text NOT NULL DEFAULT 'in_flight',
    amount             numeric(20,6) NOT NULL,
    currency           text NOT NULL,
    provider_reference text NULL,
    error              text NULL,
    created_at         timestamptz NOT NULL DEFAULT now(),
    updated_at         timestamptz NOT NULL DEFAULT now(),
    deleted_at         timestamptz NULL,

    CONSTRAINT billing_attempt_state_known
        CHECK (state IN ('in_flight', 'succeeded', 'failed', 'unknown'))
);

CREATE INDEX IF NOT EXISTS idx_billing_attempt_invoice ON platform.billing_attempt (invoice_id, created_at DESC);

-- Stranded attempts, for the convergence sweep. Partial, because the sweep only ever asks
-- for the two unsettled states and they are a vanishing fraction of the table.
CREATE INDEX IF NOT EXISTS idx_billing_attempt_unsettled
    ON platform.billing_attempt (created_at)
    WHERE state IN ('in_flight', 'unknown') AND deleted_at IS NULL;

-- Dunning state on the subscription.
--
-- `past_due` did not exist: the shipped CHECK allowed only active, grace and cancelled, so
-- the ladder ADR-017 decision 4 describes had no rung between "paying" and "in grace".
ALTER TABLE platform.subscription DROP CONSTRAINT IF EXISTS subscription_status_known;
ALTER TABLE platform.subscription ADD CONSTRAINT subscription_status_known
    CHECK (status IN ('active', 'past_due', 'grace', 'suspended', 'cancelled'));

-- How many collection attempts this billing period has already burned, and when the next
-- one is due. Both are reset to 0/NULL when an invoice is paid, which is what makes the
-- ladder reversible rather than a one-way trip to suspension.
ALTER TABLE platform.subscription
    ADD COLUMN IF NOT EXISTS dunning_attempts integer NOT NULL DEFAULT 0;
ALTER TABLE platform.subscription
    ADD COLUMN IF NOT EXISTS next_attempt_at timestamptz NULL;

-- The billing anchor: where this merchant's period boundaries fall.
--
-- Without it every merchant would be billed on the same calendar day, which makes the
-- billing run a thundering herd against bKash and puts every merchant's dunning on the
-- same schedule. NULL means "not yet billed"; the first invoice sets it.
ALTER TABLE platform.subscription
    ADD COLUMN IF NOT EXISTS current_period_start timestamptz NULL;
