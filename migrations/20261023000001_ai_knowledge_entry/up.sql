-- ADR-032 phase 32.1: tenant-owned merchant policy and FAQ knowledge.
-- Stores merchant-authored static content only; customer conversations and live commerce
-- data remain in their owning modules and are never copied into this corpus.

CREATE SCHEMA IF NOT EXISTS ai;
GRANT USAGE ON SCHEMA ai TO stheno_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA ai TO stheno_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA ai
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO stheno_app;

CREATE TABLE ai.knowledge_entry (
    id             text        PRIMARY KEY,
    tenant_id      text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    title          text        NOT NULL,
    category       text,
    content        text        NOT NULL,
    search_vector  tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('simple', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('simple', coalesce(category, '')), 'B') ||
        setweight(to_tsvector('simple', coalesce(content, '')), 'C')
    ) STORED,
    created_at     timestamptz NOT NULL DEFAULT now(),
    updated_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at     timestamptz
);

-- serves: tenant-scoped OKF retrieval
CREATE INDEX ai_knowledge_entry_search_idx
    ON ai.knowledge_entry USING gin (search_vector)
    WHERE deleted_at IS NULL;

-- serves: admin duplicate detection for an active title/category pair
CREATE UNIQUE INDEX ai_knowledge_entry_active_title_category_uq
    ON ai.knowledge_entry (tenant_id, lower(btrim(title)), coalesce(lower(btrim(category)), ''))
    WHERE deleted_at IS NULL;

ALTER TABLE ai.knowledge_entry ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.knowledge_entry FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON ai.knowledge_entry
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
