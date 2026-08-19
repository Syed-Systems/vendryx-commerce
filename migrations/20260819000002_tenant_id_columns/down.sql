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
        EXECUTE format('ALTER TABLE %I.%I DROP COLUMN IF EXISTS tenant_id',
                       t.schemaname, t.tablename);
    END LOOP;
END $$;
