-- Migration for stheno-order schema (DATA_MODEL §2.4, TASKS P5-01, P13-14)
CREATE SCHEMA IF NOT EXISTS "order";

-- 1. Address
CREATE TABLE IF NOT EXISTS "order"."address" (
    id VARCHAR(255) PRIMARY KEY,
    customer_id VARCHAR(255),
    company VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    address_1 VARCHAR(255),
    address_2 VARCHAR(255),
    city VARCHAR(255),
    country_code VARCHAR(10),
    province VARCHAR(255),
    postal_code VARCHAR(50),
    phone VARCHAR(50),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 2. Order
CREATE TABLE IF NOT EXISTS "order"."order" (
    id VARCHAR(255) PRIMARY KEY,
    display_id BIGINT NOT NULL,
    region_id VARCHAR(255),
    customer_id VARCHAR(255),
    sales_channel_id VARCHAR(255),
    email VARCHAR(255),
    currency_code VARCHAR(10) NOT NULL,
    shipping_address_id VARCHAR(255) REFERENCES "order"."address"(id) ON DELETE SET NULL,
    billing_address_id VARCHAR(255) REFERENCES "order"."address"(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    payment_status VARCHAR(50) NOT NULL DEFAULT 'not_paid',
    fulfillment_status VARCHAR(50) NOT NULL DEFAULT 'not_fulfilled',
    version INT NOT NULL DEFAULT 1,
    is_draft_order BOOLEAN NOT NULL DEFAULT FALSE,
    metadata JSONB,
    canceled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders?display_id=
CREATE UNIQUE INDEX IF NOT EXISTS idx_order_display_id ON "order"."order" (display_id) WHERE deleted_at IS NULL;

-- serves: GET /store/orders?customer_id=
CREATE INDEX IF NOT EXISTS idx_order_customer_created ON "order"."order" (customer_id, created_at DESC) WHERE deleted_at IS NULL;

-- serves: GET /admin/orders?status=
CREATE INDEX IF NOT EXISTS idx_order_status_created ON "order"."order" (status, created_at DESC) WHERE deleted_at IS NULL;

-- 3. LineItem
CREATE TABLE IF NOT EXISTS "order"."line_item" (
    id VARCHAR(255) PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    subtitle VARCHAR(255),
    thumbnail VARCHAR(1024),
    quantity INT NOT NULL,
    unit_price NUMERIC(20, 4) NOT NULL,
    is_tax_inclusive BOOLEAN NOT NULL DEFAULT FALSE,
    compare_at_unit_price NUMERIC(20, 4),
    requires_shipping BOOLEAN NOT NULL DEFAULT TRUE,
    is_discountable BOOLEAN NOT NULL DEFAULT TRUE,
    is_custom_price BOOLEAN NOT NULL DEFAULT FALSE,
    variant_id VARCHAR(255),
    product_id VARCHAR(255),
    product_title VARCHAR(255),
    product_description TEXT,
    product_subtitle VARCHAR(255),
    product_type VARCHAR(255),
    product_collection VARCHAR(255),
    product_handle VARCHAR(255),
    variant_sku VARCHAR(255),
    variant_barcode VARCHAR(255),
    variant_title VARCHAR(255),
    variant_option_values JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 4. OrderItem (Versioned Snapshot)
CREATE TABLE IF NOT EXISTS "order"."order_item" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    version INT NOT NULL,
    quantity INT NOT NULL,
    fulfilled_quantity INT NOT NULL DEFAULT 0,
    shipped_quantity INT NOT NULL DEFAULT 0,
    return_requested_quantity INT NOT NULL DEFAULT 0,
    return_received_quantity INT NOT NULL DEFAULT 0,
    return_dismissed_quantity INT NOT NULL DEFAULT 0,
    written_off_quantity INT NOT NULL DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}?version=
CREATE INDEX IF NOT EXISTS idx_order_item_order_version ON "order"."order_item" (order_id, version);

-- 5. LineItemAdjustment
CREATE TABLE IF NOT EXISTS "order"."line_item_adjustment" (
    id VARCHAR(255) PRIMARY KEY,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    code VARCHAR(255),
    amount NUMERIC(20, 4) NOT NULL,
    description TEXT,
    promotion_id VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 6. LineItemTaxLine
CREATE TABLE IF NOT EXISTS "order"."line_item_tax_line" (
    id VARCHAR(255) PRIMARY KEY,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    code VARCHAR(255) NOT NULL,
    rate NUMERIC(10, 4) NOT NULL,
    provider_id VARCHAR(255),
    tax_rate_id VARCHAR(255),
    description TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 7. ShippingMethod
CREATE TABLE IF NOT EXISTS "order"."shipping_method" (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    amount NUMERIC(20, 4) NOT NULL,
    is_tax_inclusive BOOLEAN NOT NULL DEFAULT FALSE,
    shipping_option_id VARCHAR(255),
    data JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 8. OrderShippingMethod
CREATE TABLE IF NOT EXISTS "order"."order_shipping_method" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    shipping_method_id VARCHAR(255) NOT NULL REFERENCES "order"."shipping_method"(id) ON DELETE CASCADE,
    version INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}/shipping-methods?version=
CREATE INDEX IF NOT EXISTS idx_order_shipping_method_version ON "order"."order_shipping_method" (order_id, version);

-- 9. ShippingMethodAdjustment
CREATE TABLE IF NOT EXISTS "order"."shipping_method_adjustment" (
    id VARCHAR(255) PRIMARY KEY,
    shipping_method_id VARCHAR(255) NOT NULL REFERENCES "order"."shipping_method"(id) ON DELETE CASCADE,
    code VARCHAR(255),
    amount NUMERIC(20, 4) NOT NULL,
    description TEXT,
    promotion_id VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 10. ShippingMethodTaxLine
CREATE TABLE IF NOT EXISTS "order"."shipping_method_tax_line" (
    id VARCHAR(255) PRIMARY KEY,
    shipping_method_id VARCHAR(255) NOT NULL REFERENCES "order"."shipping_method"(id) ON DELETE CASCADE,
    code VARCHAR(255) NOT NULL,
    rate NUMERIC(10, 4) NOT NULL,
    provider_id VARCHAR(255),
    tax_rate_id VARCHAR(255),
    description TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 11. OrderSummary
CREATE TABLE IF NOT EXISTS "order"."order_summary" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    version INT NOT NULL,
    paid_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    refunded_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    credit_line_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    raw_difference NUMERIC(20, 4) NOT NULL DEFAULT 0,
    pending_difference NUMERIC(20, 4) NOT NULL DEFAULT 0,
    current_order_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    original_order_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    accounting_total NUMERIC(20, 4) NOT NULL DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}/summary?version=
CREATE INDEX IF NOT EXISTS idx_order_summary_version ON "order"."order_summary" (order_id, version);

-- 12. OrderChange
CREATE TABLE IF NOT EXISTS "order"."order_change" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    version INT NOT NULL,
    change_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    requested_by VARCHAR(255),
    requested_at TIMESTAMPTZ,
    confirmed_by VARCHAR(255),
    confirmed_at TIMESTAMPTZ,
    declined_by VARCHAR(255),
    declined_at TIMESTAMPTZ,
    canceled_by VARCHAR(255),
    canceled_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}/changes
CREATE INDEX IF NOT EXISTS idx_order_change_order_status ON "order"."order_change" (order_id, status);

-- 13. OrderChangeAction
CREATE TABLE IF NOT EXISTS "order"."order_change_action" (
    id VARCHAR(255) PRIMARY KEY,
    order_change_id VARCHAR(255) NOT NULL REFERENCES "order"."order_change"(id) ON DELETE CASCADE,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    reference VARCHAR(255),
    reference_id VARCHAR(255),
    amount NUMERIC(20, 4),
    applied BOOLEAN NOT NULL DEFAULT FALSE,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}/timeline
CREATE INDEX IF NOT EXISTS idx_order_change_action_timeline ON "order"."order_change_action" (order_change_id, created_at);

-- 14. Transaction
CREATE TABLE IF NOT EXISTS "order"."transaction" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    amount NUMERIC(20, 4) NOT NULL,
    currency_code VARCHAR(10) NOT NULL,
    reference VARCHAR(100) NOT NULL,
    reference_id VARCHAR(255) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/orders/{id}/transactions
CREATE INDEX IF NOT EXISTS idx_order_transaction_order ON "order"."transaction" (order_id, created_at);

-- 15. CreditLine
CREATE TABLE IF NOT EXISTS "order"."credit_line" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    reference VARCHAR(100),
    reference_id VARCHAR(255),
    amount NUMERIC(20, 4) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 16. ReturnReason
CREATE TABLE IF NOT EXISTS "order"."return_reason" (
    id VARCHAR(255) PRIMARY KEY,
    value VARCHAR(255) NOT NULL UNIQUE,
    label VARCHAR(255) NOT NULL,
    description TEXT,
    parent_return_reason_id VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 17. Return
CREATE TABLE IF NOT EXISTS "order"."return" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    claim_id VARCHAR(255),
    exchange_id VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'requested',
    refund_amount NUMERIC(20, 4),
    location_id VARCHAR(255),
    requested_at TIMESTAMPTZ,
    received_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/returns?order_id=
CREATE INDEX IF NOT EXISTS idx_return_order ON "order"."return" (order_id, created_at);

-- 18. ReturnItem
CREATE TABLE IF NOT EXISTS "order"."return_item" (
    id VARCHAR(255) PRIMARY KEY,
    return_id VARCHAR(255) NOT NULL REFERENCES "order"."return"(id) ON DELETE CASCADE,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    quantity INT NOT NULL,
    received_quantity INT NOT NULL DEFAULT 0,
    damaged_quantity INT NOT NULL DEFAULT 0,
    reason_id VARCHAR(255) REFERENCES "order"."return_reason"(id) ON DELETE SET NULL,
    note TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 19. Claim
CREATE TABLE IF NOT EXISTS "order"."claim" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    order_change_id VARCHAR(255) REFERENCES "order"."order_change"(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'not_paid',
    refund_amount NUMERIC(20, 4),
    requested_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    created_by VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/claims?order_id=
CREATE INDEX IF NOT EXISTS idx_claim_order ON "order"."claim" (order_id, created_at);

-- 20. ClaimItem
CREATE TABLE IF NOT EXISTS "order"."claim_item" (
    id VARCHAR(255) PRIMARY KEY,
    claim_id VARCHAR(255) NOT NULL REFERENCES "order"."claim"(id) ON DELETE CASCADE,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    quantity INT NOT NULL,
    reason VARCHAR(100) NOT NULL,
    raw_quantity JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 21. ClaimItemImage
CREATE TABLE IF NOT EXISTS "order"."claim_item_image" (
    id VARCHAR(255) PRIMARY KEY,
    claim_item_id VARCHAR(255) NOT NULL REFERENCES "order"."claim_item"(id) ON DELETE CASCADE,
    url VARCHAR(1024) NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 22. Exchange
CREATE TABLE IF NOT EXISTS "order"."exchange" (
    id VARCHAR(255) PRIMARY KEY,
    order_id VARCHAR(255) NOT NULL REFERENCES "order"."order"(id) ON DELETE CASCADE,
    order_change_id VARCHAR(255) REFERENCES "order"."order_change"(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'not_paid',
    difference_due NUMERIC(20, 4),
    allow_backorder BOOLEAN NOT NULL DEFAULT FALSE,
    created_by VARCHAR(255),
    canceled_at TIMESTAMPTZ,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- serves: GET /admin/exchanges?order_id=
CREATE INDEX IF NOT EXISTS idx_exchange_order ON "order"."exchange" (order_id, created_at);

-- 23. ExchangeItem
CREATE TABLE IF NOT EXISTS "order"."exchange_item" (
    id VARCHAR(255) PRIMARY KEY,
    exchange_id VARCHAR(255) NOT NULL REFERENCES "order"."exchange"(id) ON DELETE CASCADE,
    item_id VARCHAR(255) NOT NULL REFERENCES "order"."line_item"(id) ON DELETE CASCADE,
    quantity INT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
