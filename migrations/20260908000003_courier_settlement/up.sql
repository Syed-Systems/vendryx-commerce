-- P24-01: Courier settlement cycle and settlement line tables.
-- Tenant-owned, RLS + FORCE per ADR-012.

CREATE TABLE IF NOT EXISTS payment.courier_settlement_cycle (
    id                  text            PRIMARY KEY,
    tenant_id           text            NOT NULL DEFAULT current_setting('app.tenant_id', true),
    provider_id         text            NOT NULL,                                  -- e.g. "fp_pathao", "fp_steadfast", "fp_redx"
    courier_cycle_id    text            NOT NULL,                                  -- courier-side payout statement / invoice reference
    cycle_start         timestamptz,
    cycle_end           timestamptz,
    total_collected     numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    total_fees          numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    total_remitted      numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    currency            text            NOT NULL DEFAULT 'bdt',
    status              text            NOT NULL DEFAULT 'open',                   -- "open", "reconciled", "discrepant", "closed"
    raw_report          jsonb,
    created_at          timestamptz     NOT NULL DEFAULT now(),
    updated_at          timestamptz     NOT NULL DEFAULT now(),
    deleted_at          timestamptz
);

-- serves: GET /admin/settlements/cycles?provider_id=&status=
CREATE INDEX IF NOT EXISTS idx_courier_settlement_cycle_lookup
    ON payment.courier_settlement_cycle (tenant_id, provider_id, created_at DESC)
    WHERE deleted_at IS NULL;

-- serves: Idempotent cycle re-import check
CREATE UNIQUE INDEX IF NOT EXISTS idx_courier_settlement_cycle_unique
    ON payment.courier_settlement_cycle (tenant_id, provider_id, courier_cycle_id)
    WHERE deleted_at IS NULL;

ALTER TABLE payment.courier_settlement_cycle ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment.courier_settlement_cycle FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON payment.courier_settlement_cycle
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

CREATE TABLE IF NOT EXISTS payment.courier_settlement_line (
    id                      text            PRIMARY KEY,
    tenant_id               text            NOT NULL DEFAULT current_setting('app.tenant_id', true),
    cycle_id                text            NOT NULL REFERENCES payment.courier_settlement_cycle(id) ON DELETE CASCADE,
    consignment_id          text            NOT NULL,                              -- tracking code or merchant order reference
    tracking_code           text,
    order_id                text,
    collected_amount        numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    courier_fee             numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    cod_fee                 numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    remitted_amount         numeric(20, 4)  NOT NULL DEFAULT 0.0000,
    reconciliation_status   text            NOT NULL DEFAULT 'unmatched',          -- "matched", "discrepant", "unmatched"
    discrepancy_reason      text,
    created_at              timestamptz     NOT NULL DEFAULT now(),
    updated_at              timestamptz     NOT NULL DEFAULT now(),
    deleted_at              timestamptz
);

-- serves: GET /admin/settlements/cycles/:id/lines
CREATE INDEX IF NOT EXISTS idx_courier_settlement_line_cycle
    ON payment.courier_settlement_line (tenant_id, cycle_id, created_at ASC)
    WHERE deleted_at IS NULL;

-- serves: GET /admin/orders/:id/settlement
CREATE INDEX IF NOT EXISTS idx_courier_settlement_line_order
    ON payment.courier_settlement_line (tenant_id, order_id)
    WHERE deleted_at IS NULL;

-- serves: Look up settlement line by tracking code during courier import
CREATE INDEX IF NOT EXISTS idx_courier_settlement_line_tracking
    ON payment.courier_settlement_line (tenant_id, tracking_code)
    WHERE deleted_at IS NULL;

-- serves: Idempotent line re-import deduplication
CREATE UNIQUE INDEX IF NOT EXISTS idx_courier_settlement_line_unique
    ON payment.courier_settlement_line (tenant_id, cycle_id, consignment_id)
    WHERE deleted_at IS NULL;

ALTER TABLE payment.courier_settlement_line ENABLE ROW LEVEL SECURITY;
ALTER TABLE payment.courier_settlement_line FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON payment.courier_settlement_line
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
