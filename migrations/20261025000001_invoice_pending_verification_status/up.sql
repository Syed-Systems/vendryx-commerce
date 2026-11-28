ALTER TABLE platform.invoice
    DROP CONSTRAINT invoice_status_known;

ALTER TABLE platform.invoice
    ADD CONSTRAINT invoice_status_known
    CHECK (status IN ('open', 'pending_verification', 'paid', 'void', 'uncollectible'));
