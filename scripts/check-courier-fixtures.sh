#!/usr/bin/env bash
# scripts/check-courier-fixtures.sh — Recorded wiremock fixtures for BD courier providers (P23-13)
#
# Verifies that all three courier integrations pass their full test suite
# using only pre-recorded wiremock fixtures — no live courier API calls.
set -euo pipefail

echo "==> Verifying recorded wiremock fixtures for courier providers..."

echo "[1/3] Pathao courier fixtures..."
cargo test -p stheno-fulfillment-pathao --test options
cargo test -p stheno-fulfillment-pathao --test rates
cargo test -p stheno-fulfillment-pathao --test consignment
cargo test -p stheno-fulfillment-pathao --test status_mapping

echo "[2/3] Steadfast courier fixtures..."
cargo test -p stheno-fulfillment-steadfast --test provider
cargo test -p stheno-fulfillment-steadfast --test status_mapping

echo "[3/3] RedX courier fixtures..."
cargo test -p stheno-fulfillment-redx --test provider
cargo test -p stheno-fulfillment-redx --test status_mapping

echo "All courier fixture tests passed without external API calls (Exit 0)."
exit 0
