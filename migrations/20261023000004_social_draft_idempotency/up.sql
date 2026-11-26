-- ADR-032 phase 32.5: idempotent AI handoff and collision-safe human reference codes.

-- serves: SocialDraftWorkflow::create_ai_social_draft lookup by source provider event
CREATE UNIQUE INDEX idx_order_social_event_uq
    ON "order"."order" (tenant_id, (metadata->>'source_event_id'))
    WHERE deleted_at IS NULL
      AND is_draft_order = true
      AND metadata->>'source' = 'social'
      AND metadata ? 'source_event_id';

-- serves: SocialDraftWorkflow retry when allocating the buyer/staff reference code
CREATE UNIQUE INDEX idx_order_social_ref_uq
    ON "order"."order" (tenant_id, (split_part(metadata->>'ref', '|', 1)))
    WHERE deleted_at IS NULL
      AND is_draft_order = true
      AND metadata->>'source' = 'social'
      AND metadata ? 'ref';
