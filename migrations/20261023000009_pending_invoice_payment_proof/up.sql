DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM platform.invoice_payment_proof
        WHERE deleted_at IS NULL AND status = 'pending'
        GROUP BY invoice_id
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'duplicate pending invoice payment proofs require manual review before applying this migration';
    END IF;
END $$;

-- serves: POST /admin/billing/invoices/{invoice_id}/payment-proof; one proof may be under review per invoice
CREATE UNIQUE INDEX idx_invoice_payment_proof_one_pending_per_invoice
    ON platform.invoice_payment_proof (invoice_id)
    WHERE deleted_at IS NULL AND status = 'pending';

CREATE FUNCTION platform.prevent_paid_invoice_reopening() RETURNS trigger AS $$
BEGIN
    IF OLD.status = 'paid' AND NEW.status = 'pending_verification' THEN
        RAISE EXCEPTION 'paid invoice cannot return to pending verification'
            USING ERRCODE = '23514';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_paid_invoice_reopening
    BEFORE UPDATE OF status ON platform.invoice
    FOR EACH ROW
    EXECUTE FUNCTION platform.prevent_paid_invoice_reopening();
