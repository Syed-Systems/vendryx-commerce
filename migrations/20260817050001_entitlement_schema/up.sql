-- Phase 15: Entitlement usage counters (ENTITLEMENTS.md §6.2)
CREATE TABLE IF NOT EXISTS entitlement_usage (
    resource text PRIMARY KEY,
    used     bigint NOT NULL DEFAULT 0
);

INSERT INTO entitlement_usage (resource, used) VALUES
    ('products', 0),
    ('users', 0),
    ('regions', 0)
ON CONFLICT (resource) DO NOTHING;
