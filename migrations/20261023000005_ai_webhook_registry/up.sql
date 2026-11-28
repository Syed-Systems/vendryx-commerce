-- ADR-019 requires a random webhook slug resolvable before tenant scope is known.
-- This table contains routing metadata only; credential ciphertext stays in ai.configuration.
CREATE TABLE ai.webhook_registry (
    webhook_id  text PRIMARY KEY,
    tenant_id   text NOT NULL,
    kind        text NOT NULL CHECK (kind IN ('whatsapp', 'messenger', 'instagram')),
    account_id  text NOT NULL,
    active      boolean NOT NULL DEFAULT false,
    updated_at  timestamptz NOT NULL DEFAULT now()
);

REVOKE ALL ON ai.webhook_registry FROM PUBLIC, stheno_app;

CREATE FUNCTION ai.register_webhook(
    p_webhook_id text,
    p_tenant_id text,
    p_kind text,
    p_account_id text,
    p_active boolean
) RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, ai
AS $$
DECLARE changed integer;
BEGIN
    INSERT INTO ai.webhook_registry (webhook_id, tenant_id, kind, account_id, active)
    VALUES (p_webhook_id, p_tenant_id, p_kind, p_account_id, p_active)
    ON CONFLICT (webhook_id) DO UPDATE
        SET active = EXCLUDED.active, updated_at = now()
        WHERE ai.webhook_registry.tenant_id = EXCLUDED.tenant_id
          AND ai.webhook_registry.kind = EXCLUDED.kind
          AND ai.webhook_registry.account_id = EXCLUDED.account_id;
    GET DIAGNOSTICS changed = ROW_COUNT;
    RETURN changed = 1;
END;
$$;

CREATE FUNCTION ai.resolve_webhook(p_webhook_id text)
RETURNS TABLE (tenant_id text, kind text, account_id text)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, ai
AS $$
    SELECT registry.tenant_id, registry.kind, registry.account_id
      FROM ai.webhook_registry registry
     WHERE registry.webhook_id = p_webhook_id AND registry.active = true
     LIMIT 1
$$;

REVOKE ALL ON FUNCTION ai.register_webhook(text, text, text, text, boolean) FROM PUBLIC;
REVOKE ALL ON FUNCTION ai.resolve_webhook(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ai.register_webhook(text, text, text, text, boolean) TO stheno_app;
GRANT EXECUTE ON FUNCTION ai.resolve_webhook(text) TO stheno_app;
