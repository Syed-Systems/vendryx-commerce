-- phase: contract
DROP TABLE IF EXISTS platform.billing_attempt;
DROP TABLE IF EXISTS platform.invoice_line;
DROP TABLE IF EXISTS platform.invoice;

ALTER TABLE platform.subscription DROP COLUMN IF EXISTS current_period_start;
ALTER TABLE platform.subscription DROP COLUMN IF EXISTS next_attempt_at;
ALTER TABLE platform.subscription DROP COLUMN IF EXISTS dunning_attempts;

ALTER TABLE platform.subscription DROP CONSTRAINT IF EXISTS subscription_status_known;
ALTER TABLE platform.subscription ADD CONSTRAINT subscription_status_known
    CHECK (status IN ('active', 'grace', 'cancelled'));
