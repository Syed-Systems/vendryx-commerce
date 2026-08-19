-- ADR-012 step 2 of 3: give every tenant-owned table a tenant_id.
--
-- Driven by a loop over pg_tables rather than ~157 hand-written ALTERs: shorter, and it
-- cannot typo a table name. Auditability is bought back by
-- crates/core/stheno-db/tests/rls_coverage.rs, which asserts coverage on every future
-- table too — the loop runs once, the test runs forever.
--
-- The DEFAULT is load-bearing: PgCrudRepository names the entity's own columns on INSERT
-- and therefore omits tenant_id, so the default is what populates it. (It only works
-- because those INSERTs name their columns — `SELECT * FROM json_populate_record` would
-- supply tenant_id explicitly as NULL and the default would never fire.)

DO $$
DECLARE
    t record;
    -- Schemas that hold tenant-owned data. `platform` is excluded because it IS the
    -- tenant registry, and gets its own narrower policies.
    app_schemas text[] := ARRAY[
        'analytics','api_key','audit','auth','caching','cart','customer','file',
        'fulfillment','index','inventory','links','notification','order','payment',
        'pricing','product','promotion','rbac','region','reporting','sales_channel',
        'search','settings','stock_location','store','tax','translation','user','workflow'
    ];
    -- The exclusion list is the leak surface. Keep it short; argue every entry.
    --   currency.currency  - ISO 4217 reference data, PK is the code itself. Two tenants
    --                        cannot disagree about what "usd" is.
    --   region.country     - excluded here and CORRECTED in 20260819000006. It carries
    --                        region_id, a tenant-owned FK, so it was never reference data.
    excluded text[] := ARRAY['currency.currency','region.country'];
BEGIN
    FOR t IN
        SELECT schemaname, tablename
          FROM pg_tables
         WHERE schemaname = ANY(app_schemas)
           AND (schemaname || '.' || tablename) <> ALL(excluded)
    LOOP
        EXECUTE format('ALTER TABLE %I.%I ADD COLUMN IF NOT EXISTS tenant_id text',
                       t.schemaname, t.tablename);
        -- Everything that exists today belongs to the default tenant.
        EXECUTE format('UPDATE %I.%I SET tenant_id = %L WHERE tenant_id IS NULL',
                       t.schemaname, t.tablename, 'ten_default');
        EXECUTE format('ALTER TABLE %I.%I ALTER COLUMN tenant_id SET NOT NULL',
                       t.schemaname, t.tablename);
        EXECUTE format($f$ALTER TABLE %I.%I ALTER COLUMN tenant_id
                          SET DEFAULT current_setting('app.tenant_id', true)$f$,
                       t.schemaname, t.tablename);
        EXECUTE format('CREATE INDEX IF NOT EXISTS %I ON %I.%I (tenant_id)',
                       'idx_' || t.tablename || '_tenant', t.schemaname, t.tablename);
    END LOOP;
END $$;

-- currency.currency is deliberately global; see the exclusion list. region.country was
-- too, wrongly, until 20260819000006.
