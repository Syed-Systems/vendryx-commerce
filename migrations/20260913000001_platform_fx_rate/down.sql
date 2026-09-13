ALTER TABLE platform.invoice
    DROP COLUMN IF EXISTS fx_metadata,
    DROP COLUMN IF EXISTS payable_amount;

DROP TABLE IF EXISTS platform.fx_rate;
