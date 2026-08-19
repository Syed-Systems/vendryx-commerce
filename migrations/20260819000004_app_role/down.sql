-- The role is cluster-scoped and may be in use by other databases; revoke this
-- database's grants rather than dropping it.
DO $$
DECLARE
    s text;
BEGIN
    FOR s IN SELECT nspname FROM pg_namespace
              WHERE nspname NOT IN ('pg_catalog','information_schema')
                AND nspname NOT LIKE 'pg_%'
    LOOP
        EXECUTE format('REVOKE ALL ON ALL TABLES IN SCHEMA %I FROM stheno_app', s);
        EXECUTE format('REVOKE ALL ON SCHEMA %I FROM stheno_app', s);
    END LOOP;
END $$;
