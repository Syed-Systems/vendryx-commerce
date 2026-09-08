#!/usr/bin/env bash
# scripts/check-provider-fixtures.sh — Recorded sandbox fixtures for BD payment providers (P22-18)
set -euo pipefail

echo "==> Verifying recorded sandbox fixtures for BD payment providers..."

echo "[1/3] Verifying bKash wiremock sandbox fixtures..."
cargo test -p stheno-payment-bkash --test bkash_flow
cargo test -p stheno-payment-bkash --test initiate
cargo test -p stheno-payment-bkash --test capture
cargo test -p stheno-payment-bkash --test refund

echo "[2/3] Verifying SSLCommerz wiremock sandbox fixtures..."
cargo test -p stheno-payment-sslcommerz --test sslcommerz_flow
cargo test -p stheno-payment-sslcommerz --test session
cargo test -p stheno-payment-sslcommerz --test ipn
cargo test -p stheno-payment-sslcommerz --test refund

echo "[3/3] Verifying Nagad wiremock sandbox fixtures..."
cargo test -p stheno-payment-nagad --test initialise
cargo test -p stheno-payment-nagad --test capture
cargo test -p stheno-payment-nagad --test refund
cargo test -p stheno-payment-nagad --test webhook

echo "All recorded sandbox fixtures passed without external calls (Exit 0)."
exit 0
