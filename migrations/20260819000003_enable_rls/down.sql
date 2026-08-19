-- The incident lever: isolation off, columns and data untouched.
DO $$
DECLARE
    t record;
    app_schemas text[] := ARRAY[
        'analytics','api_key','audit','auth','caching','cart','customer','file',
        'fulfillment','index','inventory','links','notification','order','payment',
        'pricing','product','promotion','rbac','region','reporting','sales_channel',
        'search','settings','stock_location','store','tax','translation','user','workflow'
    ];
BEGIN
    FOR t IN
        SELECT schemaname, tablename FROM pg_tables WHERE schemaname = ANY(app_schemas)
    LOOP
        EXECUTE format('DROP POLICY IF EXISTS tenant_isolation ON %I.%I',
                       t.schemaname, t.tablename);
        EXECUTE format('ALTER TABLE %I.%I NO FORCE ROW LEVEL SECURITY',
                       t.schemaname, t.tablename);
        EXECUTE format('ALTER TABLE %I.%I DISABLE ROW LEVEL SECURITY',
                       t.schemaname, t.tablename);
    END LOOP;
END $$;

DROP POLICY IF EXISTS tenant_isolation ON entitlement_usage;
ALTER TABLE entitlement_usage NO FORCE ROW LEVEL SECURITY;
ALTER TABLE entitlement_usage DISABLE ROW LEVEL SECURITY;
