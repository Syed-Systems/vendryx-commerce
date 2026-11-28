-- ADR-035: tenant-owned recommendation decisions and append-only review history.
CREATE TABLE ai.store_intelligence_review (
    tenant_id          text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    recommendation_id text        NOT NULL CHECK (length(recommendation_id) BETWEEN 1 AND 255),
    decision           text        NOT NULL CHECK (decision IN ('approved', 'dismissed', 'reverted')),
    actor_id           text        NOT NULL,
    version            integer     NOT NULL DEFAULT 1 CHECK (version > 0),
    changed_at         timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, recommendation_id)
);

CREATE TABLE ai.store_intelligence_review_audit (
    id                  text        PRIMARY KEY,
    tenant_id           text        NOT NULL DEFAULT current_setting('app.tenant_id', true),
    recommendation_id  text        NOT NULL CHECK (length(recommendation_id) BETWEEN 1 AND 255),
    request_id          text        NOT NULL CHECK (length(request_id) BETWEEN 1 AND 64),
    decision            text        NOT NULL CHECK (decision IN ('approved', 'dismissed', 'reverted')),
    actor_id            text        NOT NULL,
    evidence_source     text        NOT NULL CHECK (evidence_source = 'tenant_order_snapshots'),
    rule_version        text        NOT NULL CHECK (rule_version = 'store-intelligence-recommendations-v1'),
    created_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, request_id)
);

-- serves: load latest merchant decisions for the bounded Store Intelligence dashboard
CREATE INDEX ai_store_intelligence_review_tenant_changed
    ON ai.store_intelligence_review (tenant_id, changed_at DESC);
-- serves: review audit by tenant and recommendation in reverse chronological order
CREATE INDEX ai_store_intelligence_review_audit_tenant_recommendation
    ON ai.store_intelligence_review_audit (tenant_id, recommendation_id, created_at DESC);

ALTER TABLE ai.store_intelligence_review ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.store_intelligence_review FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.store_intelligence_review
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
ALTER TABLE ai.store_intelligence_review_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai.store_intelligence_review_audit FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ai.store_intelligence_review_audit
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
GRANT SELECT, INSERT, UPDATE ON ai.store_intelligence_review TO stheno_app;
GRANT SELECT, INSERT ON ai.store_intelligence_review_audit TO stheno_app;
