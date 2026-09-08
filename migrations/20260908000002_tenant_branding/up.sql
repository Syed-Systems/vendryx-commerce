-- P26-02: Tenant branding table.
-- Stores per-tenant customisation: store name, logo, favicon, font key, colour tokens.
-- One row per tenant; absent row yields documented defaults at the API layer.
-- Tenant-owned, RLS + FORCE per ADR-012.

CREATE TABLE IF NOT EXISTS store.tenant_branding (
    id              text        PRIMARY KEY,
    tenant_id       text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    -- Display identity
    store_name      text,                                   -- overrides the default store name
    -- Asset references (file module IDs, not raw URLs)
    logo_file_id    text,                                   -- SVG or PNG, validated at upload
    favicon_file_id text,                                   -- ICO, PNG or SVG, validated at upload
    -- Typography: a key from the curated font set (P26-04)
    font_key        text,                                   -- e.g. "inter", "noto_sans_bengali"
    -- Colour tokens: CSS custom property values (hex or rgba string)
    -- The storefront resolves these into --brand-* CSS variables (P26-05).
    color_primary   text,                                   -- e.g. "#1a56db"
    color_secondary text,
    color_accent    text,
    color_background text,
    color_surface   text,
    color_text      text,
    -- Audit
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz
);

-- One row per tenant, no duplicates.
-- serves: GET /store/branding, admin branding endpoint
CREATE UNIQUE INDEX IF NOT EXISTS tenant_branding_tenant_uq
    ON store.tenant_branding (tenant_id)
    WHERE deleted_at IS NULL;

-- serves: admin branding page lookup
CREATE INDEX IF NOT EXISTS tenant_branding_tenant_idx
    ON store.tenant_branding (tenant_id)
    WHERE deleted_at IS NULL;

ALTER TABLE store.tenant_branding ENABLE ROW LEVEL SECURITY;
ALTER TABLE store.tenant_branding FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON store.tenant_branding
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
