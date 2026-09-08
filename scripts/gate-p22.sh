#!/usr/bin/env bash
# scripts/gate-p22.sh — Phase gate for P22: Bangladesh Payment Gateways (PHASE3_TASKS.md P22)
set -euo pipefail

echo "=========================================================="
echo "          P22 PHASE GATE: BANGLADESH PAYMENT GATEWAYS     "
echo "=========================================================="

echo "[1/8] Universal gates: Module isolation, boundaries, and kernel purity..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-schema-drift.sh
./scripts/check-provider-pci.sh
python scripts/check-no-fabricated-handlers.py

echo "[2/8] P22-01..P22-06: bKash Tokenized Checkout suite..."
cargo test -p stheno-payment-bkash --test registration
cargo test -p stheno-payment-bkash --test token_lifecycle
cargo test -p stheno-payment-bkash --test initiate
cargo test -p stheno-payment-bkash --test capture
cargo test -p stheno-payment-bkash --test webhook
cargo test -p stheno-payment-bkash --test refund

echo "[3/8] P22-07..P22-11: Nagad DFS Checkout suite..."
cargo test -p stheno-payment-nagad --test registration
cargo test -p stheno-payment-nagad --test initialise
cargo test -p stheno-payment-nagad --test capture
cargo test -p stheno-payment-nagad --test webhook
cargo test -p stheno-payment-nagad --test refund

echo "[4/8] P22-12..P22-15: SSLCommerz Hosted Session & IPN suite..."
cargo test -p stheno-payment-sslcommerz --test registration
cargo test -p stheno-payment-sslcommerz --test session
cargo test -p stheno-payment-sslcommerz --test ipn
cargo test -p stheno-payment-sslcommerz --test refund

echo "[5/8] P22-16: All three providers pass provider conformance unmodified..."
./scripts/check-provider-conformance.sh

echo "[6/8] P22-17: BDT end to end: decimal totals & minor-unit conversion invariants..."
cargo test -p stheno-payment --test bdt_money

echo "[7/8] P22-18: Recorded wiremock sandbox fixtures verification..."
./scripts/check-provider-fixtures.sh

echo "[8/8] Workspace build and compiler hygiene..."
cargo check --workspace

echo "=========================================================="
echo "STATUS: GATE P22 PASSED (Exit 0)"
echo "=========================================================="
exit 0
