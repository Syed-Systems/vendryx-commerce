DROP TRIGGER IF EXISTS prevent_paid_invoice_reopening ON platform.invoice;
DROP FUNCTION IF EXISTS platform.prevent_paid_invoice_reopening();
DROP INDEX IF EXISTS platform.idx_invoice_payment_proof_one_pending_per_invoice;
