DROP INDEX IF EXISTS index.index_sync_state_entity_name_key;
CREATE UNIQUE INDEX index_sync_state_entity_name_key
    ON index.index_sync_state (entity_name);

DROP INDEX IF EXISTS promotion.campaign_campaign_identifier_key;
CREATE UNIQUE INDEX campaign_campaign_identifier_key
    ON promotion.campaign (campaign_identifier);

DROP INDEX IF EXISTS "order".idx_order_display_id;
CREATE UNIQUE INDEX idx_order_display_id
    ON "order"."order" (display_id) WHERE deleted_at IS NULL;

DROP TABLE IF EXISTS "order".display_id_counter;
