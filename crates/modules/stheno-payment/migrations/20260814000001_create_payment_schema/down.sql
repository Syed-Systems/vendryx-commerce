-- Phase: contract
-- Drops the payment module schema and all owned tables.

DROP TABLE IF EXISTS payment.payment_reconciliation_run;
DROP TABLE IF EXISTS payment.payment_webhook_event;
DROP TABLE IF EXISTS payment.payment_provider_request;
DROP TABLE IF EXISTS payment.account_holder;
DROP TABLE IF EXISTS payment.refund;
DROP TABLE IF EXISTS payment.capture;
DROP TABLE IF EXISTS payment.refund_reason;
DROP TABLE IF EXISTS payment.payment;
DROP TABLE IF EXISTS payment.payment_session;
DROP TABLE IF EXISTS payment.payment_provider;
DROP TABLE IF EXISTS payment.payment_collection;

DROP SCHEMA IF EXISTS payment;
