-- An outbound request ledger for couriers, so a timed-out consignment is recoverable.
--
-- payment.payment_provider_request has existed since P4-18 and gives payments a durable
-- record written *before* the provider call, which is the only thing that makes an
-- ambiguous timeout recoverable. Fulfillment had no equivalent, so
-- `create_fulfillment` was the one money-moving call in the system with no crash-safety
-- at all.
--
-- The failure that motivates it is worse for couriers than for gateways. A gateway
-- timeout risks a lost payment; a courier timeout risks a *duplicate parcel* — retry
-- the create, get two consignments, and in a COD market that is two deliveries and two
-- cash collections against one order. Neither the customer nor the merchant finds out
-- until someone knocks on the door twice.
--
-- Tenant-owned: a merchant's consignment requests are theirs. RLS + FORCE, because the
-- application owns this table and Postgres exempts owners from RLS unless it is forced
-- (see rls_audit).
CREATE TABLE IF NOT EXISTS fulfillment.fulfillment_provider_request (
    id              text PRIMARY KEY,
    -- The repository writes entity structs and omits tenant_id, so the default is what
    -- populates it (same pattern as 20260819000002). Without it every insert lands NULL
    -- and the policy below rejects it.
    tenant_id       text NOT NULL DEFAULT current_setting('app.tenant_id', true),
    idempotency_key text NOT NULL,
    provider_id     text NOT NULL,
    operation       text NOT NULL,
    resource_type   text NOT NULL,
    resource_id     text NOT NULL,
    request_hash    text NOT NULL,
    -- in_flight -> a call was started and never settled (the crash shape)
    -- unknown    -> the call returned ambiguously (the timeout shape)
    -- succeeded / failed -> decided
    state           text NOT NULL DEFAULT 'in_flight',
    response        jsonb,
    error           jsonb,
    attempts        integer NOT NULL DEFAULT 1,
    first_sent_at   timestamptz NOT NULL DEFAULT now(),
    settled_at      timestamptz,
    deleted_at      timestamptz
);

-- The dedupe guarantee. Scoped by tenant so two merchants cannot collide on a key
-- either of them chose.
CREATE UNIQUE INDEX IF NOT EXISTS fulfillment_provider_request_key_uq
    ON fulfillment.fulfillment_provider_request (tenant_id, idempotency_key)
    WHERE deleted_at IS NULL;

-- The sweep's access path: find everything still undecided, oldest first.
CREATE INDEX IF NOT EXISTS fulfillment_provider_request_unsettled_idx
    ON fulfillment.fulfillment_provider_request (state, first_sent_at)
    WHERE deleted_at IS NULL;

ALTER TABLE fulfillment.fulfillment_provider_request ENABLE ROW LEVEL SECURITY;
ALTER TABLE fulfillment.fulfillment_provider_request FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON fulfillment.fulfillment_provider_request
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
