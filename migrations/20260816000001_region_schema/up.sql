-- Phase: expand
-- Creates the region module schema (DATA_MODEL 1).

CREATE SCHEMA IF NOT EXISTS region;

CREATE TABLE IF NOT EXISTS region.region (
    id              text PRIMARY KEY,
    name            text NOT NULL,
    currency_code   text NOT NULL,
    automatic_taxes boolean NOT NULL DEFAULT true,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz NULL
);

-- serves: GET /store/regions and admin region lists, which exclude soft-deleted rows
CREATE INDEX IF NOT EXISTS idx_region_currency_code
    ON region.region (currency_code) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS region.country (
    id           text PRIMARY KEY,
    iso_2        text NOT NULL,
    iso_3        text NOT NULL,
    num_code     text NOT NULL,
    name         text NOT NULL,
    display_name text NOT NULL,
    region_id    text NULL REFERENCES region.region (id) ON DELETE SET NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now(),
    deleted_at   timestamptz NULL
);

-- A country belongs to at most one region, enforced by the database rather than
-- by a read-then-write check in the service.
CREATE UNIQUE INDEX IF NOT EXISTS idx_country_iso_2 ON region.country (iso_2);
-- serves: resolving a region's countries on every region read
CREATE INDEX IF NOT EXISTS idx_country_region_id ON region.country (region_id);
