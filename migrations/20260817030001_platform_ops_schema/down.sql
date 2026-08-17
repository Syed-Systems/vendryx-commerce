-- Migration: 20260817030001_platform_ops_schema
-- Down Migration: Drop schemas for caching, index, search, analytics, audit, translation, settings, notification, file

DROP SCHEMA IF EXISTS caching CASCADE;
DROP SCHEMA IF EXISTS "index" CASCADE;
DROP SCHEMA IF EXISTS search CASCADE;
DROP SCHEMA IF EXISTS analytics CASCADE;
DROP SCHEMA IF EXISTS audit CASCADE;
DROP SCHEMA IF EXISTS translation CASCADE;
DROP SCHEMA IF EXISTS settings CASCADE;
DROP SCHEMA IF EXISTS notification CASCADE;
DROP SCHEMA IF EXISTS "file" CASCADE;
