DROP TABLE IF EXISTS rbac.audit_record;
DROP TABLE IF EXISTS rbac.policy;
DROP TABLE IF EXISTS rbac.role_inheritance;
DROP TABLE IF EXISTS rbac."role";
DROP SCHEMA IF EXISTS rbac CASCADE;

DROP TABLE IF EXISTS api_key.api_key;
DROP SCHEMA IF EXISTS api_key CASCADE;

DROP TABLE IF EXISTS "user"."invite";
DROP TABLE IF EXISTS "user"."user";
DROP SCHEMA IF EXISTS "user" CASCADE;

DROP TABLE IF EXISTS auth.password_reset_token;
DROP TABLE IF EXISTS auth.mfa_factor;
DROP TABLE IF EXISTS auth.token_family;
DROP TABLE IF EXISTS auth.auth_session;
DROP TABLE IF EXISTS auth.auth_provider;
DROP TABLE IF EXISTS auth.auth_identity;
DROP SCHEMA IF EXISTS auth CASCADE;

DROP TABLE IF EXISTS customer.customer_address;
DROP TABLE IF EXISTS customer.customer_group;
DROP TABLE IF EXISTS customer.customer;
DROP SCHEMA IF EXISTS customer CASCADE;
