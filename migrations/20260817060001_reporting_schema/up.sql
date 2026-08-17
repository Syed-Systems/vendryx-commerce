-- Phase 16: Reporting schema and daily rollups (PHASE2_PLAN.md §P16)
CREATE SCHEMA IF NOT EXISTS reporting;

-- 1. daily_rollup
CREATE TABLE IF NOT EXISTS reporting.daily_rollup (
    date date NOT NULL,
    currency text NOT NULL,
    order_count bigint NOT NULL DEFAULT 0,
    total_revenue numeric(20, 4) NOT NULL DEFAULT 0,
    units_sold bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    PRIMARY KEY (date, currency)
);

CREATE INDEX IF NOT EXISTS idx_reporting_daily_rollup_date ON reporting.daily_rollup (date DESC) WHERE deleted_at IS NULL;

-- 2. daily_product_rollup
CREATE TABLE IF NOT EXISTS reporting.daily_product_rollup (
    date date NOT NULL,
    product_id text NOT NULL,
    title text NOT NULL,
    units_sold bigint NOT NULL DEFAULT 0,
    total_revenue numeric(20, 4) NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    PRIMARY KEY (date, product_id)
);

CREATE INDEX IF NOT EXISTS idx_reporting_product_rollup_date ON reporting.daily_product_rollup (date DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_reporting_product_rollup_product ON reporting.daily_product_rollup (product_id) WHERE deleted_at IS NULL;
