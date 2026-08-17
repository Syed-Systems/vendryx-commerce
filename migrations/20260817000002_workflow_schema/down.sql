-- Phase: contract
-- Reverts the workflow schema and its owned tables.

DROP TABLE IF EXISTS workflow.lease CASCADE;
DROP TABLE IF EXISTS workflow.step_state CASCADE;
DROP TABLE IF EXISTS workflow.execution CASCADE;
DROP SCHEMA IF EXISTS workflow CASCADE;
