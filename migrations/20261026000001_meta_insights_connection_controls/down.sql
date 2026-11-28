DELETE FROM ai.configuration
WHERE kind IN ('meta_insights_facebook', 'meta_insights_instagram');
DELETE FROM ai.store_intelligence_source_observation
WHERE source IN ('facebook_page', 'instagram_account');
DELETE FROM ai.source_connection_audit
WHERE source IN ('facebook_page', 'instagram_account');

ALTER TABLE ai.configuration
    DROP CONSTRAINT configuration_kind_check;
ALTER TABLE ai.configuration
    ADD CONSTRAINT configuration_kind_check
    CHECK (kind IN ('llm', 'whatsapp', 'messenger', 'instagram', 'search_console'));

ALTER TABLE ai.source_connection_audit
    DROP CONSTRAINT source_connection_audit_source_check;
ALTER TABLE ai.source_connection_audit
    ADD CONSTRAINT source_connection_audit_source_check
    CHECK (source = 'search_console');

REVOKE DELETE ON ai.source_connection_audit FROM stheno_app;
REVOKE DELETE ON ai.store_intelligence_review FROM stheno_app;
REVOKE DELETE ON ai.store_intelligence_review_audit FROM stheno_app;
