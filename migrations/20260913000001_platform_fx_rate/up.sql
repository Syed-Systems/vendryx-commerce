CREATE TABLE IF NOT EXISTS platform.fx_rate (
    id             text PRIMARY KEY,
    currency_pair  text NOT NULL UNIQUE,
    rate           numeric(20,6) NOT NULL,
    source         text NOT NULL,
    rate_kind      text NOT NULL,
    operator_id    text NULL REFERENCES platform.operator(id) ON DELETE SET NULL,
    rate_date      timestamptz NOT NULL,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz NULL
);

ALTER TABLE platform.invoice
    ADD COLUMN fx_metadata jsonb NULL,
    ADD COLUMN payable_amount numeric(20,6) NULL;
