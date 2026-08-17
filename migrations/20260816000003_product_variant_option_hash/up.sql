-- Phase: expand
-- Persists variant option-value identity (P11-19).
--
-- The hash previously lived only in a `variant_hashes` HashMap beside the
-- service, so variant identity was lost on restart and uniqueness was enforced
-- by a read-then-write check that two concurrent writers can both pass.
-- Storing it as a column lets the database enforce it (P12-13 removes the
-- remaining read-then-write path entirely).

ALTER TABLE product.product_variant
    ADD COLUMN IF NOT EXISTS option_values_hash text NOT NULL DEFAULT '';

-- Identity is per product: two different products may each have a "Small / Red"
-- variant, but one product may not have two.
CREATE UNIQUE INDEX IF NOT EXISTS idx_variant_product_option_hash
    ON product.product_variant (product_id, option_values_hash)
    WHERE deleted_at IS NULL;
