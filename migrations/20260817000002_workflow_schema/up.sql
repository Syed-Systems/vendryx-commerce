-- Phase: expand
-- Creates the workflow schema and tables for execution tracking, step states, and worker leases.

CREATE SCHEMA IF NOT EXISTS workflow;

-- 1. workflow.execution
CREATE TABLE IF NOT EXISTS workflow.execution (
    id text PRIMARY KEY,
    workflow_name text NOT NULL,
    state text NOT NULL DEFAULT 'NotStarted',
    step_states jsonb NOT NULL DEFAULT '{}'::jsonb,
    context_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    lease_owner text NULL,
    lease_expires_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: worker claim loop selecting active/invoking workflows
CREATE INDEX IF NOT EXISTS idx_workflow_execution_state ON workflow.execution (state) WHERE deleted_at IS NULL;
-- serves: lease recovery queries
CREATE INDEX IF NOT EXISTS idx_workflow_execution_lease ON workflow.execution (lease_expires_at) WHERE lease_expires_at IS NOT NULL AND deleted_at IS NULL;

-- 2. workflow.step_state
CREATE TABLE IF NOT EXISTS workflow.step_state (
    id text PRIMARY KEY,
    transaction_id text NOT NULL REFERENCES workflow.execution(id) ON DELETE CASCADE,
    step_key text NOT NULL,
    step_name text NOT NULL,
    step_index integer NOT NULL DEFAULT 0,
    state text NOT NULL DEFAULT 'Pending',
    input jsonb NULL,
    output jsonb NULL,
    error text NULL,
    attempts integer NOT NULL DEFAULT 0,
    compensation_input jsonb NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: step state lookup by workflow execution and step key
CREATE UNIQUE INDEX IF NOT EXISTS idx_workflow_step_state_tx_key ON workflow.step_state (transaction_id, step_key);
CREATE INDEX IF NOT EXISTS idx_workflow_step_state_tx_id ON workflow.step_state (transaction_id) WHERE deleted_at IS NULL;

-- 3. workflow.lease
CREATE TABLE IF NOT EXISTS workflow.lease (
    id text PRIMARY KEY,
    transaction_id text NOT NULL UNIQUE REFERENCES workflow.execution(id) ON DELETE CASCADE,
    owner_id text NOT NULL,
    lease_expires_at timestamptz NOT NULL,
    renewed_at timestamptz NOT NULL DEFAULT now(),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    deleted_at timestamptz NULL
);

-- serves: lease expiration scanning
CREATE INDEX IF NOT EXISTS idx_workflow_lease_expiry ON workflow.lease (lease_expires_at) WHERE deleted_at IS NULL;
