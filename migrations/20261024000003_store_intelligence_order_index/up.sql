-- serves: GET /admin/ai/store-intelligence tenant-scoped rolling order-window aggregate
CREATE INDEX idx_order_store_intelligence_tenant_created
    ON "order"."order" (tenant_id, created_at DESC)
    WHERE deleted_at IS NULL
      AND canceled_at IS NULL
      AND is_draft_order = FALSE
      AND status <> 'canceled';
