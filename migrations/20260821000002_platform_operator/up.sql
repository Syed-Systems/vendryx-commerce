-- Operator identity and an audit trail for the control plane.
--
-- Until now `/platform/*` was gated on one shared `STHENO_PLATFORM_TOKEN`. That is
-- workable for a single operator and indefensible for two: there is no record of who
-- moved a merchant between plans, no way to revoke one person without revoking everyone,
-- and no rotation that is not an outage. The merchant dashboard already has RBAC and MFA;
-- the surface that can read and rewrite *every* merchant had neither.
--
-- Global, not tenant-scoped, for the same reason as platform.tier_definition: `platform`
-- is outside the RLS schema list (20260819000003), and an operator belongs to the
-- deployment rather than to a merchant. A tenant-scoped credential table would put the
-- cross-tenant credential inside one tenant's row set, and would make the audit trail
-- unreadable across tenants — which is the only way it is worth reading.

CREATE TABLE IF NOT EXISTS platform.operator (
    id              text PRIMARY KEY,
    name            text NOT NULL,
    -- SHA-256 of the issued token. The token is 32 random bytes, so it is not
    -- guessable and does not need a password KDF; this matches api_key.api_key,
    -- which hashes its secrets the same way.
    token_hash      text NOT NULL UNIQUE,
    -- `op_...abcd`. Enough to tell two credentials apart in the UI, not enough to use.
    redacted_token  text NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    last_used_at    timestamptz NULL,
    -- Revocation is a timestamp, not a delete: the audit rows an operator wrote still
    -- name them, and a row that vanished would leave those pointing at nothing.
    revoked_at      timestamptz NULL,
    -- Always NULL. `PgCrudRepository::retrieve` predicates on `deleted_at IS NULL`
    -- unconditionally, so a table it retrieves from must carry the column; `Operator`
    -- reports `is_deleted() == false` and never writes it. Revocation is `revoked_at`.
    deleted_at      timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_operator_live ON platform.operator (token_hash) WHERE revoked_at IS NULL;

-- Every write the control plane makes, and who made it.
--
-- Deliberately not audit.audit_log: that table is tenant-scoped and RLS-forced, so a
-- cross-tenant read of it is impossible by design — and "show me everything this operator
-- changed" is exactly a cross-tenant read. Rows here are also about the deployment
-- (redefining a plan) as often as about one merchant.
--
-- Append-only by construction: the service exposes no update and no delete, and there is
-- deliberately no `deleted_at` here — the column that lets every other table be soft-
-- deleted is exactly what an audit trail must not have. Only `create` and `list` are ever
-- called against it; `retrieve` would fail loudly on the missing column, which is correct.
CREATE TABLE IF NOT EXISTS platform.operator_audit (
    id           text PRIMARY KEY,
    -- The operator id, or `op_bootstrap` for the environment token.
    actor_id     text NOT NULL,
    -- Denormalised so the trail still reads after a credential is renamed or revoked.
    actor_name   text NOT NULL,
    action       text NOT NULL,
    entity_type  text NOT NULL,
    entity_id    text NOT NULL,
    -- What changed. Never a credential: the issued token is not written here.
    diff         jsonb NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);

-- Indexed on `id DESC`, not `created_at DESC`, because that is the order the trail is
-- both read and paged in. `id` is a ULID: its leading bits are the timestamp, so it sorts
-- by time like `created_at` does, and unlike `created_at` it is unique — which is what
-- makes `WHERE id < $cursor` a cursor that can neither skip a row nor repeat one when
-- several rows share a millisecond.
CREATE INDEX IF NOT EXISTS idx_operator_audit_recent ON platform.operator_audit (id DESC);
CREATE INDEX IF NOT EXISTS idx_operator_audit_entity ON platform.operator_audit (entity_id, id DESC);
CREATE INDEX IF NOT EXISTS idx_operator_audit_actor ON platform.operator_audit (actor_id, id DESC);
