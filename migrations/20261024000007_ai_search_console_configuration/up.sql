-- ADR-035: allow tenant-bound encrypted credentials for the read-only Search Console adapter.
ALTER TABLE ai.configuration
    DROP CONSTRAINT configuration_kind_check,
    ADD CONSTRAINT configuration_kind_check
        CHECK (kind IN ('llm', 'whatsapp', 'messenger', 'instagram', 'search_console'));
