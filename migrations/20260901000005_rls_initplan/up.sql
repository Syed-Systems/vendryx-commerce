-- Make the RLS predicate evaluate `current_setting` once per QUERY, not once per ROW.
--
-- 20260819000003 wrote the policy as:
--     USING (tenant_id = current_setting('app.tenant_id', true))
--
-- Wrapping the call in a scalar subquery turns it into an InitPlan: evaluated once,
-- cached for the statement, then compared against the column like any constant.
--
--     USING (tenant_id = (SELECT current_setting('app.tenant_id', true)))
--
-- # What this actually buys, measured rather than assumed
--
-- Measured on Postgres 16, 200k rows, 50 tenants, `tenant_id` indexed:
--
--   | path                        | bare              | wrapped  |
--   |-----------------------------|-------------------|----------|
--   | index-eligible equality     | 2.5 ms            | 3.2 ms   |
--   | seq scan (index disabled)   | 36.3 ms (parallel)| 18.3 ms  |
--
-- **On the indexed path there is no win.** `current_setting` is a built-in STABLE
-- function and the planner already hoists it into the Index Cond, so the widely-cited
-- advice about wrapping policy functions (written for user-defined ones like Supabase's
-- `auth.uid()`) does not transfer to this predicate. Anyone expecting orders of magnitude
-- here should stop and re-measure rather than assume the change regressed.
--
-- The win is on scans the index cannot drive: roughly 2x, because the bare form evaluates
-- `current_setting` once per row examined. Note the bare plan above went *parallel* and
-- still lost to the single-threaded wrapped one. Reporting rollups, admin list endpoints
-- with unindexed filters, and anything doing a full pass over a large tenant's table land
-- on that path.
--
-- So: a real but bounded win, at zero risk and zero maintenance. Kept for that reason,
-- not because it is dramatic.
--
-- This is safe only because the value does not depend on row data, which is the stated
-- precondition for the optimisation and holds here by construction — the tenant is set
-- once per transaction by `stheno_db::tenant` before any statement runs.
--
-- Isolation semantics are UNCHANGED. `missing_ok = true` still yields NULL when unset,
-- the predicate is still NULL, and an unscoped query still returns zero rows. This
-- migration alters only when the setting is read, never what it means.
--
-- # Why this loops over pg_policies rather than over pg_tables
--
-- 20260819000003 looped over `pg_tables` and applied an exclusion list. Repeating that
-- shape here would re-derive which tables *should* have a policy, and could therefore
-- disagree with reality — creating a policy on a table deliberately left global, or
-- missing one added by a later migration. Six migrations now create `tenant_isolation`
-- policies (20260819000003, ...006, ...008, 20260821000001, ...004) across app schemas
-- and `public.entitlement_usage`.
--
-- Driving off `pg_policies` instead rewrites exactly the policies that exist, no more and
-- no fewer. A table with no policy stays without one; a table that gained one keeps it.
-- The set of protected tables is not a decision this migration is entitled to make.

DO $$
DECLARE
    p record;
BEGIN
    FOR p IN
        SELECT schemaname, tablename
          FROM pg_policies
         WHERE policyname = 'tenant_isolation'
    LOOP
        EXECUTE format('DROP POLICY tenant_isolation ON %I.%I',
                       p.schemaname, p.tablename);
        EXECUTE format($f$CREATE POLICY tenant_isolation ON %I.%I
                          USING      (tenant_id = (SELECT current_setting('app.tenant_id', true)))
                          WITH CHECK (tenant_id = (SELECT current_setting('app.tenant_id', true)))$f$,
                       p.schemaname, p.tablename);
    END LOOP;
END $$;
