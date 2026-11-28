-- Refuse rollback while encrypted Search Console credentials remain configured.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM ai.configuration WHERE kind = 'search_console') THEN
        RAISE EXCEPTION 'Cannot roll back Search Console configuration while credentials remain; disconnect each property first.';
    END IF;
END $$;

ALTER TABLE ai.configuration
    DROP CONSTRAINT configuration_kind_check,
    ADD CONSTRAINT configuration_kind_check
        CHECK (kind IN ('llm', 'whatsapp', 'messenger', 'instagram'));
