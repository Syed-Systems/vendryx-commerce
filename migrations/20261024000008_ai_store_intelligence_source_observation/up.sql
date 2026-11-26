-- ADR-035: retain bounded, tenant-scoped source observation history for comparable windows.
CREATE TABLE ai.store_intelligence_source_observation (
    tenant_id          text           NOT NULL DEFAULT current_setting('app.tenant_id', true),
    source             text           NOT NULL CHECK (length(source) BETWEEN 1 AND 64),
    account_id         text           NOT NULL CHECK (length(account_id) BETWEEN 1 AND 512),
    scope              text           NOT NULL CHECK (length(scope) BETWEEN 1 AND 128),
    metric             text           NOT NULL CHECK (length(metric) BETWEEN 1 AND 128),
    definition         text           NOT NULL CHECK (length(definition) BETWEEN 1 AND 128),
    unit               text           NOT NULL CHECK (length(unit) BETWEEN 1 AND 64),
    value              numeric        NOT NULL CHECK (value >= 0),
    denominator        numeric        CHECK (denominator IS NULL OR denominator >= 0),
    sample_size        bigint         CHECK (sample_size IS NULL OR sample_size > 0),
    window_started_on  date           NOT NULL,
    window_ended_on    date           NOT NULL CHECK (window_ended_on > window_started_on),
    window_timezone    text           NOT NULL CHECK (length(window_timezone) BETWEEN 1 AND 64),
    ingested_at        timestamptz    NOT NULL,
    coverage           text           NOT NULL CHECK (coverage IN ('complete', 'partial', 'unavailable')),
    PRIMARY KEY (tenant_id, source, account_id, metric, window_started_on, window_ended_on)
);

-- serves: GET /admin/ai/search-console/snapshots ordered by newest ingestion for one tenant/source/property
CREATE INDEX idx_ai_source_observation_recent
    ON ai.store_intelligence_source_observation (tenant_id, source, account_id, ingested_at DESC);

ALTER TABLE ai.store_intelligence_source_observation ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.store_intelligence_source_observation FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.store_intelligence_source_observation
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT, UPDATE, DELETE ON ai.store_intelligence_source_observation TO stheno_app;
