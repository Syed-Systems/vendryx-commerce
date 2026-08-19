-- ADR-012 step 3 of 3: turn isolation on.
--
-- Kept separate from step 2 so that `down.sql` here disables RLS in seconds without
-- dropping 157 columns. If isolation ever has to come off in an incident, this is the
-- lever.
--
-- FORCE is not optional. The application connects as the role that owns these tables, and
-- Postgres exempts owners from RLS unless forced — without FORCE every policy below is
-- inert and every test passes while proving nothing.
--
-- `current_setting('app.tenant_id', true)` uses missing_ok = true deliberately. Unset ->
-- NULL -> the predicate is NULL -> the row is filtered. An unscoped query therefore
-- returns ZERO ROWS and an unscoped insert is rejected. Raising instead would turn every
-- unscoped query into a 500; the Rust layer already fails loudly via tenant::current(),
-- so the database's job here is to be the quiet, safe backstop.

DO $$
DECLARE
    t record;
    app_schemas text[] := ARRAY[
        'analytics','api_key','audit','auth','caching','cart','customer','file',
        'fulfillment','index','inventory','links','notification','order','payment',
        'pricing','product','promotion','rbac','region','reporting','sales_channel',
        'search','settings','stock_location','store','tax','translation','user','workflow'
    ];
    -- region.country is excluded here and given RLS in 20260819000006 instead.
    excluded text[] := ARRAY['currency.currency','region.country'];
BEGIN
    FOR t IN
        SELECT schemaname, tablename
          FROM pg_tables
         WHERE schemaname = ANY(app_schemas)
           AND (schemaname || '.' || tablename) <> ALL(excluded)
    LOOP
        EXECUTE format('ALTER TABLE %I.%I ENABLE ROW LEVEL SECURITY',
                       t.schemaname, t.tablename);
        EXECUTE format('ALTER TABLE %I.%I FORCE ROW LEVEL SECURITY',
                       t.schemaname, t.tablename);
        EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON %I.%I',
                       t.schemaname, t.tablename);
        EXECUTE format($f$CREATE POLICY tenant_isolation ON %I.%I
                          USING      (tenant_id = current_setting('app.tenant_id', true))
                          WITH CHECK (tenant_id = current_setting('app.tenant_id', true))$f$,
                       t.schemaname, t.tablename);
    END LOOP;
END $$;

-- entitlement_usage lives in public and is scoped the same way.
ALTER TABLE entitlement_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE entitlement_usage FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON entitlement_usage;
CREATE POLICY tenant_isolation ON entitlement_usage
    USING      (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
