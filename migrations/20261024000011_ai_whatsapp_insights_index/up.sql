-- ADR-035: bound consent-filtered WhatsApp message aggregation by tenant, account, and event time.
-- serves: GET /admin/ai/whatsapp-insights processed event window aggregate
CREATE INDEX idx_ai_channel_event_whatsapp_insights_window
    ON ai.channel_event (tenant_id, external_account_id, provider_created_at)
    WHERE channel = 'whatsapp' AND state = 'processed';
