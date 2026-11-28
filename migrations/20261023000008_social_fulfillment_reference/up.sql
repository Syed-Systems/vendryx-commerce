-- serves: resolve social-order fulfillment before staff-confirmed courier dispatch
CREATE INDEX IF NOT EXISTS fulfillment_data_reference_idx
    ON fulfillment.fulfillment (tenant_id, (data ->> 'reference'))
    WHERE deleted_at IS NULL;
