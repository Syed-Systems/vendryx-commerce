CREATE TABLE platform.invoice_payment_proof (
    id VARCHAR NOT NULL PRIMARY KEY,
    invoice_id VARCHAR NOT NULL REFERENCES platform.invoice(id),
    tenant_id VARCHAR NOT NULL,
    payment_method VARCHAR NOT NULL,
    transaction_id VARCHAR NOT NULL,
    proof_asset_url VARCHAR NOT NULL,
    operator_id VARCHAR,
    reviewed_at TIMESTAMPTZ,
    status VARCHAR NOT NULL DEFAULT 'pending',
    rejection_reason VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_invoice_payment_proof_unique 
ON platform.invoice_payment_proof(tenant_id, payment_method, transaction_id) 
WHERE deleted_at IS NULL AND status != 'rejected';
