-- ADR-012: the role the application actually queries as.
--
-- WHY THIS EXISTS AT ALL, and why it is not a "later, before production" concern:
-- Postgres exempts SUPERUSERS from row-level security unconditionally — FORCE ROW LEVEL
-- SECURITY does not apply to them. The default DATABASE_URL connects as `stheno`, which
-- docker-compose creates as a superuser. Without this migration every policy added by
-- 20260819000003 is INERT: a cross-tenant read succeeds, and the whole test suite passes
-- while proving nothing. This was observed, not theorised.
--
-- WHY `SET ROLE` RATHER THAN A SECOND LOGIN:
-- A separate login role would mean a second credential to provision, rotate and inject
-- into every environment and every test database. Instead the role is NOLOGIN and the
-- application switches into it per transaction with `SET LOCAL ROLE`, which reverts on
-- COMMIT. One connection string, and RLS applies identically in dev, CI and production.
--
-- The honest limit: a superuser connection can `RESET ROLE`. This is a guard against
-- accident and missing scoping, not against hostile code already running as superuser.
-- A genuinely separate login role remains the stronger posture for production and is a
-- drop-in change — point DATABASE_URL at it and this migration keeps working.

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'stheno_app') THEN
        CREATE ROLE stheno_app NOLOGIN NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE;
    END IF;
END $$;

-- The migrating/owning role must be a member to be able to SET ROLE into it.
DO $$ BEGIN
    EXECUTE format('GRANT stheno_app TO %I', current_user);
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$
DECLARE
    s text;
    app_schemas text[] := ARRAY[
        'analytics','api_key','audit','auth','caching','cart','currency','customer','file',
        'fulfillment','index','inventory','links','notification','order','payment',
        'pricing','product','promotion','rbac','region','reporting','sales_channel',
        'search','settings','stock_location','store','tax','translation','user','workflow',
        'platform','public'
    ];
BEGIN
    FOREACH s IN ARRAY app_schemas LOOP
        IF EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = s) THEN
            EXECUTE format('GRANT USAGE ON SCHEMA %I TO stheno_app', s);
            EXECUTE format(
                'GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA %I TO stheno_app', s);
            EXECUTE format(
                'GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA %I TO stheno_app', s);
            -- Tables added by future migrations are covered without editing this file.
            EXECUTE format(
                'ALTER DEFAULT PRIVILEGES IN SCHEMA %I
                 GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO stheno_app', s);
            EXECUTE format(
                'ALTER DEFAULT PRIVILEGES IN SCHEMA %I
                 GRANT USAGE, SELECT ON SEQUENCES TO stheno_app', s);
        END IF;
    END LOOP;
END $$;
