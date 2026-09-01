-- Restore the unwrapped predicate from 20260819000003.
--
-- Same loop, same set of policies. Isolation is identical either way; reverting costs
-- per-row evaluation of `current_setting`, it does not weaken the policy.

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
                          USING      (tenant_id = current_setting('app.tenant_id', true))
                          WITH CHECK (tenant_id = current_setting('app.tenant_id', true))$f$,
                       p.schemaname, p.tablename);
    END LOOP;
END $$;
