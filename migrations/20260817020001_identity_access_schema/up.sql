-- Customer Schema
CREATE SCHEMA IF NOT EXISTS customer;

CREATE TABLE customer.customer (
    id text PRIMARY KEY,
    email text NOT NULL,
    default_billing_address_id text NULL,
    default_shipping_address_id text NULL,
    company_name text NULL,
    first_name text NULL,
    last_name text NULL,
    phone text NULL,
    has_account boolean NOT NULL DEFAULT false,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/customers?email=
CREATE INDEX idx_customer_email ON customer.customer(email) WHERE deleted_at IS NULL;

CREATE TABLE customer.customer_group (
    id text PRIMARY KEY,
    name text NOT NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE customer.customer_address (
    id text PRIMARY KEY,
    customer_id text NOT NULL,
    company text NULL,
    first_name text NULL,
    last_name text NULL,
    address_1 text NOT NULL,
    address_2 text NULL,
    city text NOT NULL,
    country_code text NOT NULL,
    province text NULL,
    postal_code text NOT NULL,
    phone text NULL,
    is_default_shipping boolean NOT NULL DEFAULT false,
    is_default_billing boolean NOT NULL DEFAULT false,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/customers/:id/addresses
CREATE INDEX idx_customer_address_customer ON customer.customer_address(customer_id) WHERE deleted_at IS NULL;

-- Auth Schema
CREATE SCHEMA IF NOT EXISTS auth;

CREATE TABLE auth.auth_identity (
    id text PRIMARY KEY,
    provider_id text NOT NULL,
    entity_id text NOT NULL,
    password_hash text NULL,
    token_version integer NOT NULL DEFAULT 1,
    actor_id text NULL,
    actor_type text NULL,
    provider_metadata jsonb NULL,
    app_metadata jsonb NULL,
    user_metadata jsonb NULL,
    failed_attempts integer NOT NULL DEFAULT 0,
    locked_until timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Auth identity lookup by provider & entity
CREATE UNIQUE INDEX idx_auth_identity_provider_entity ON auth.auth_identity(provider_id, entity_id) WHERE deleted_at IS NULL;

CREATE TABLE auth.auth_provider (
    id text PRIMARY KEY,
    name text NOT NULL,
    is_enabled boolean NOT NULL DEFAULT true,
    config jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE auth.auth_session (
    id text PRIMARY KEY,
    identity_id text NOT NULL,
    token_family_id text NOT NULL,
    token_version integer NOT NULL DEFAULT 1,
    refresh_token_hash text NOT NULL,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Session lookup by identity
CREATE INDEX idx_auth_session_identity ON auth.auth_session(identity_id);
-- serves: Session family invalidation
CREATE INDEX idx_auth_session_family ON auth.auth_session(token_family_id);

CREATE TABLE auth.token_family (
    id text PRIMARY KEY,
    identity_id text NOT NULL,
    latest_refresh_token_hash text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    revoked_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Token family lookup by identity
CREATE INDEX idx_token_family_identity ON auth.token_family(identity_id);

CREATE TABLE auth.mfa_factor (
    id text PRIMARY KEY,
    identity_id text NOT NULL,
    factor_type text NOT NULL,
    secret_encrypted text NOT NULL,
    recovery_codes_hash text[] NOT NULL DEFAULT '{}',
    is_verified boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: MFA factor lookup by identity
CREATE INDEX idx_mfa_factor_identity ON auth.mfa_factor(identity_id);

CREATE TABLE auth.password_reset_token (
    id text PRIMARY KEY,
    identity_id text NOT NULL,
    token_hash text NOT NULL,
    expires_at timestamptz NOT NULL,
    used_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Password reset token verification
CREATE INDEX idx_password_reset_token_hash ON auth.password_reset_token(token_hash);

-- User Schema
CREATE SCHEMA IF NOT EXISTS "user";

CREATE TABLE "user"."user" (
    id text PRIMARY KEY,
    email text NOT NULL,
    first_name text NULL,
    last_name text NULL,
    avatar_url text NULL,
    role text NOT NULL DEFAULT 'member',
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/users?email=
CREATE UNIQUE INDEX idx_user_email ON "user"."user"(email) WHERE deleted_at IS NULL;

CREATE TABLE "user"."invite" (
    id text PRIMARY KEY,
    email text NOT NULL,
    role text NOT NULL DEFAULT 'member',
    token text NOT NULL,
    expires_at timestamptz NOT NULL,
    accepted_at timestamptz NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: GET /admin/invites?token=
CREATE INDEX idx_invite_token ON "user"."invite"(token) WHERE deleted_at IS NULL;

-- API Key Schema
CREATE SCHEMA IF NOT EXISTS api_key;

CREATE TABLE api_key.api_key (
    id text PRIMARY KEY,
    key_type text NOT NULL,
    title text NOT NULL,
    redacted_key text NOT NULL,
    token_hash text NOT NULL,
    sales_channel_ids text[] NOT NULL DEFAULT '{}',
    permissions text[] NOT NULL DEFAULT '{}',
    created_by text NULL,
    revoked_by text NULL,
    revoked_at timestamptz NULL,
    last_used_at timestamptz NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: API key authentication lookup
CREATE INDEX idx_api_key_token_hash ON api_key.api_key(token_hash) WHERE deleted_at IS NULL AND revoked_at IS NULL;

-- RBAC Schema
CREATE SCHEMA IF NOT EXISTS rbac;

CREATE TABLE rbac."role" (
    id text PRIMARY KEY,
    name text NOT NULL,
    description text NULL,
    is_system_role boolean NOT NULL DEFAULT false,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Role lookup by name
CREATE UNIQUE INDEX idx_role_name ON rbac."role"(name) WHERE deleted_at IS NULL;

CREATE TABLE rbac.role_inheritance (
    parent_role text NOT NULL,
    child_role text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL,
    PRIMARY KEY (parent_role, child_role)
);

CREATE TABLE rbac.policy (
    id text PRIMARY KEY,
    "role" text NOT NULL,
    resource text NOT NULL,
    "action" text NOT NULL,
    effect text NOT NULL DEFAULT 'allow',
    is_sensitive boolean NOT NULL DEFAULT false,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Policy evaluation by role and resource
CREATE INDEX idx_policy_role ON rbac.policy("role");

CREATE TABLE rbac.audit_record (
    id text PRIMARY KEY,
    actor_id text NOT NULL,
    "role" text NOT NULL,
    "action" text NOT NULL,
    resource text NOT NULL,
    mfa_verified boolean NOT NULL DEFAULT false,
    decision text NOT NULL,
    "timestamp" timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: Audit log query by actor
CREATE INDEX idx_rbac_audit_actor ON rbac.audit_record(actor_id, "timestamp" DESC);
