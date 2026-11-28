-- ADR-035: keep insights credentials separate from messaging/webhook credentials.
ALTER TABLE ai.configuration
    DROP CONSTRAINT configuration_kind_check;
ALTER TABLE ai.configuration
    ADD CONSTRAINT configuration_kind_check
    CHECK (kind IN (
        'llm', 'whatsapp', 'messenger', 'instagram',
        'search_console', 'meta_insights_facebook', 'meta_insights_instagram'
    ));

ALTER TABLE ai.source_connection_audit
    DROP CONSTRAINT source_connection_audit_source_check;
ALTER TABLE ai.source_connection_audit
    ADD CONSTRAINT source_connection_audit_source_check
    CHECK (source IN ('search_console', 'facebook_page', 'instagram_account'));

GRANT DELETE ON ai.source_connection_audit TO stheno_app;
GRANT DELETE ON ai.store_intelligence_review TO stheno_app;
GRANT DELETE ON ai.store_intelligence_review_audit TO stheno_app;
