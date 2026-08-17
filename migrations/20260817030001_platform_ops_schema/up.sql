-- Migration: 20260817030001_platform_ops_schema
-- Up Migration: Full schemas for file, notification, settings, translation, audit, analytics, search, index, caching

-- 1. File Schema
CREATE SCHEMA IF NOT EXISTS "file";

CREATE TABLE IF NOT EXISTS "file"."file" (
    id text PRIMARY KEY,
    filename text NOT NULL,
    url text NOT NULL,
    mime_type text NOT NULL,
    size_bytes bigint NOT NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_file_filename ON "file"."file" (filename) WHERE deleted_at IS NULL;

-- 2. Notification Schema
CREATE SCHEMA IF NOT EXISTS notification;

CREATE TABLE IF NOT EXISTS notification.notification (
    id text PRIMARY KEY,
    "to" text NOT NULL,
    channel text NOT NULL,
    template text NULL,
    data jsonb NULL,
    trigger_type text NULL,
    resource_id text NULL,
    resource_type text NULL,
    receiver_id text NULL,
    status text NOT NULL DEFAULT 'pending',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS notification.notification_template (
    id text PRIMARY KEY,
    name text NOT NULL,
    channel text NOT NULL,
    subject_template text NULL,
    body_template text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS notification.notification_delivery (
    id text PRIMARY KEY,
    notification_id text NOT NULL,
    provider_id text NOT NULL,
    delivered_at timestamptz NULL,
    error_message text NULL,
    metadata jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_notification_to ON notification.notification ("to") WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_notification_channel ON notification.notification (channel) WHERE deleted_at IS NULL;

-- 3. Settings Schema
CREATE SCHEMA IF NOT EXISTS settings;

CREATE TABLE IF NOT EXISTS settings.store_setting (
    id text PRIMARY KEY,
    key text NOT NULL UNIQUE,
    value jsonb NOT NULL,
    category text NOT NULL,
    is_secret boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS settings.view_config (
    id text PRIMARY KEY,
    user_id text NULL,
    scope text NOT NULL,
    columns jsonb NOT NULL,
    filters jsonb NULL,
    sort_by text NULL,
    sort_order text NULL,
    page_size bigint NOT NULL DEFAULT 20,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS settings.property_label (
    id text PRIMARY KEY,
    entity_type text NOT NULL,
    property_key text NOT NULL,
    label text NOT NULL,
    help_text text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS settings.layout_preference (
    id text PRIMARY KEY,
    user_id text NOT NULL,
    layout_key text NOT NULL,
    config jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- 4. Translation Schema
CREATE SCHEMA IF NOT EXISTS translation;

CREATE TABLE IF NOT EXISTS translation.locale (
    id text PRIMARY KEY,
    code text NOT NULL UNIQUE,
    name text NOT NULL,
    is_default boolean NOT NULL DEFAULT false,
    is_enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS translation.translation (
    id text PRIMARY KEY,
    locale_code text NOT NULL,
    entity_type text NOT NULL,
    entity_id text NOT NULL,
    field_name text NOT NULL,
    value text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS translation.translation_override (
    id text PRIMARY KEY,
    translation_id text NOT NULL,
    sales_channel_id text NULL,
    region_id text NULL,
    override_value text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_trans_entity ON translation.translation (entity_type, entity_id, locale_code) WHERE deleted_at IS NULL;

-- 5. Audit Schema
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS audit.audit_log (
    id text PRIMARY KEY,
    actor_id text NOT NULL,
    actor_type text NOT NULL,
    action text NOT NULL,
    entity_type text NOT NULL,
    entity_id text NOT NULL,
    diff jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit.audit_log (actor_id, created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit.audit_log (entity_type, entity_id, created_at DESC) WHERE deleted_at IS NULL;

-- 6. Analytics Schema
CREATE SCHEMA IF NOT EXISTS analytics;

CREATE TABLE IF NOT EXISTS analytics.analytics_event (
    id text PRIMARY KEY,
    event_name text NOT NULL,
    distinct_id text NOT NULL,
    properties jsonb NOT NULL,
    timestamp timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_analytics_event_name ON analytics.analytics_event (event_name, timestamp DESC) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_analytics_distinct_id ON analytics.analytics_event (distinct_id, timestamp DESC) WHERE deleted_at IS NULL;

-- 7. Search Schema
CREATE SCHEMA IF NOT EXISTS search;

CREATE TABLE IF NOT EXISTS search.search_document (
    id text PRIMARY KEY,
    index_name text NOT NULL,
    document_id text NOT NULL,
    document jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_search_doc ON search.search_document (index_name, document_id) WHERE deleted_at IS NULL;

-- 8. Index Schema
CREATE SCHEMA IF NOT EXISTS "index";

CREATE TABLE IF NOT EXISTS "index".index_data (
    id text PRIMARY KEY,
    entity_name text NOT NULL,
    entity_id text NOT NULL,
    data jsonb NOT NULL,
    version bigint NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS "index".index_relation (
    id text PRIMARY KEY,
    parent_entity text NOT NULL,
    parent_id text NOT NULL,
    child_entity text NOT NULL,
    child_id text NOT NULL,
    relation_type text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS "index".index_field (
    id text PRIMARY KEY,
    entity_name text NOT NULL,
    field_name text NOT NULL,
    field_type text NOT NULL,
    is_searchable boolean NOT NULL DEFAULT true,
    is_filterable boolean NOT NULL DEFAULT true,
    is_sortable boolean NOT NULL DEFAULT true,
    deleted_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS "index".index_sync_state (
    id text PRIMARY KEY,
    entity_name text NOT NULL UNIQUE,
    last_synced_version bigint NOT NULL DEFAULT 0,
    last_synced_at timestamptz NOT NULL DEFAULT now(),
    lag_seconds bigint NOT NULL DEFAULT 0,
    status text NOT NULL DEFAULT 'synced',
    deleted_at timestamptz NULL
);

CREATE INDEX IF NOT EXISTS idx_index_data_entity ON "index".index_data (entity_name, entity_id) WHERE deleted_at IS NULL;

-- 9. Caching Schema
CREATE SCHEMA IF NOT EXISTS caching;

CREATE TABLE IF NOT EXISTS caching.cache_entry (
    key text PRIMARY KEY,
    value jsonb NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);
