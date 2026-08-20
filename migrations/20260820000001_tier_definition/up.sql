-- Plan definitions become data, so retuning a plan does not need a release.
--
-- ENTITLEMENTS.md 5b put tier definitions in `config/tiers.toml` and said adding or
-- retuning a tier must never require a release. That holds for a self-hosted install
-- where an operator can edit a file; it does not hold for the hosted deployment ADR-012
-- introduced, where the config is baked into a container image and the operator is
-- looking at a browser. This table is the writable layer: a row, when present, REPLACES
-- that tier's definition wholesale. No row means the shipped TOML still governs, so an
-- untouched deployment behaves exactly as before.
--
-- Replacement rather than merge is deliberate. A merge needs an answer for "the row omits
-- `max_products` — is that unlimited, or unspecified?", and every answer to that question
-- is a plan silently granting more than the operator believes it does.
--
-- Global, not tenant-scoped: `platform` is outside the RLS schema list (20260819000003),
-- and a tier means the same thing to every merchant. Which tier a merchant is ON stays in
-- platform.subscription, which is where per-tenant divergence belongs.
CREATE TABLE IF NOT EXISTS platform.tier_definition (
    tier        text PRIMARY KEY,
    features    jsonb NOT NULL DEFAULT '[]'::jsonb,
    quotas      jsonb NOT NULL DEFAULT '{}'::jsonb,
    windows     jsonb NOT NULL DEFAULT '{}'::jsonb,
    updated_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT tier_definition_known CHECK (tier IN ('trial', 'free', 'pro'))
);

-- Per-tenant feature grants and revocations.
--
-- quota_overrides and window_overrides already existed (20260819000001); features were
-- the one axis of an entitlement with no per-tenant lever, so "give this one merchant the
-- payment gateway" had no answer short of moving them to Pro and giving away the rest of
-- it. Shape is {"payment_gateway": true, "fraud_score": false} — an explicit false is a
-- revocation, and an absent key means the tier decides.
ALTER TABLE platform.subscription
    ADD COLUMN IF NOT EXISTS feature_overrides jsonb NOT NULL DEFAULT '{}'::jsonb;
