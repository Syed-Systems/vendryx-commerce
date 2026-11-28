DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM platform.invoice
        WHERE status = 'pending_verification'
    ) THEN
        RAISE EXCEPTION 'pending verification invoices must be reviewed before rolling back this migration';
    END IF;
END $$;

ALTER TABLE platform.invoice
    DROP CONSTRAINT invoice_status_known;

ALTER TABLE platform.invoice
    ADD CONSTRAINT invoice_status_known
    CHECK (status IN ('open', 'paid', 'void', 'uncollectible'));
