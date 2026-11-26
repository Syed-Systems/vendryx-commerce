-- ADR-034: shopper-submitted manual payment references remain pending until staff review.
CREATE TABLE "order".payment_proof (
    id text PRIMARY KEY,
    tenant_id text NOT NULL,
    order_id text NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    payment_method text NOT NULL CHECK (payment_method IN ('bkash', 'nagad', 'rocket', 'bank_transfer')),
    transaction_id text NOT NULL CHECK (length(transaction_id) BETWEEN 1 AND 128),
    proof_asset_url text NULL CHECK (proof_asset_url IS NULL OR length(proof_asset_url) <= 2048),
    status text NOT NULL DEFAULT 'pending_verification'
        CHECK (status IN ('pending_verification', 'verified', 'rejected')),
    submitted_at timestamptz NOT NULL DEFAULT now(),
    reviewed_at timestamptz NULL,
    reviewed_by text NULL,
    review_note text NULL CHECK (review_note IS NULL OR length(review_note) <= 1000),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE "order".payment_proof ENABLE ROW LEVEL SECURITY;
ALTER TABLE "order".payment_proof FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON "order".payment_proof
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

-- serves: concurrent duplicate-proof rejection for POST /store/orders/{id}/payment-proof
CREATE UNIQUE INDEX order_payment_proof_one_pending_per_order
    ON "order".payment_proof (tenant_id, order_id)
    WHERE status = 'pending_verification';
