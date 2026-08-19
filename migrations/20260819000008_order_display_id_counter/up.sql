-- `order.display_id` — the human-readable order number — was neither per-tenant nor durable.
--
-- Two faults, and each hid the other:
--
-- 1. It came from `Arc<AtomicI64>` initialised to 1000 in `OrderService::new`. Process
--    memory. Every restart began numbering at 1000 again, so the first order after a
--    restart collided with an order already stored. The section it sits under is titled
--    "Gap-Free Sequential display_id Generation" and it was neither across a restart.
-- 2. `idx_order_display_id` was globally unique. Since the counter was shared by the whole
--    process the tenants never raced *each other* within one run, which is why this looked
--    fine — until a restart, when the collision surfaced as a duplicate-key error on
--    checkout and on every social draft.
--
-- A counter row per tenant fixes both. `UPDATE ... RETURNING` takes a row lock, so
-- concurrent order creation in one tenant serialises on that row rather than racing to read
-- the same MAX and having one of them fail — which, on an order placement, is the worst
-- possible place for a retry.

CREATE TABLE IF NOT EXISTS "order".display_id_counter (
    tenant_id  text PRIMARY KEY,
    next_value bigint NOT NULL DEFAULT 1
);

-- Existing tenants resume above their highest order, so no stored order number is reused.
INSERT INTO "order".display_id_counter (tenant_id, next_value)
SELECT tenant_id, MAX(display_id) + 1
  FROM "order"."order"
 GROUP BY tenant_id
ON CONFLICT (tenant_id) DO UPDATE SET next_value = EXCLUDED.next_value;

ALTER TABLE "order".display_id_counter ENABLE ROW LEVEL SECURITY;
ALTER TABLE "order".display_id_counter FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS tenant_isolation ON "order".display_id_counter;
CREATE POLICY tenant_isolation ON "order".display_id_counter
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

-- Order numbers are per merchant. Two merchants both having an order #1000 is normal;
-- the global index made the second one impossible.
DROP INDEX IF EXISTS "order".idx_order_display_id;
CREATE UNIQUE INDEX idx_order_display_id
    ON "order"."order" (tenant_id, display_id) WHERE deleted_at IS NULL;

-- Same argument, two more natural keys missed by 20260819000005.
-- A campaign identifier is merchant-chosen text; two merchants may both run "BLACKFRIDAY".
-- The constraint owns the index, so it goes first; dropping the index alone errors.
ALTER TABLE promotion.campaign DROP CONSTRAINT IF EXISTS campaign_campaign_identifier_key;
DROP INDEX IF EXISTS promotion.campaign_campaign_identifier_key;
CREATE UNIQUE INDEX campaign_campaign_identifier_key
    ON promotion.campaign (tenant_id, campaign_identifier);

-- `entity_name` is a type name like 'product'. Every tenant has one sync-state row per
-- entity, so this was global by accident.
ALTER TABLE index.index_sync_state DROP CONSTRAINT IF EXISTS index_sync_state_entity_name_key;
DROP INDEX IF EXISTS index.index_sync_state_entity_name_key;
CREATE UNIQUE INDEX index_sync_state_entity_name_key
    ON index.index_sync_state (tenant_id, entity_name);
