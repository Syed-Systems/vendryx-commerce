#!/usr/bin/env bash
# scripts/gate-p24.sh — Phase gate for P24: COD settlement and reconciliation (PHASE3_TASKS.md P24)
set -euo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo "=========================================================="
echo "          P24 PHASE GATE: COD SETTLEMENT & RECONCILIATION "
echo "=========================================================="

echo "[1/10] Universal gates: Module isolation, boundaries, and kernel purity..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-schema-drift.sh
python scripts/check-no-fabricated-handlers.py

echo "[2/10] P24-01: Courier settlement tables RLS audit..."
cargo test -p stheno-db --test rls_audit

echo "[3/10] P24-02: Order three distinct amounts (order total, collected, remitted)..."
cargo test -p stheno-order --test cod_amounts

echo "[4/10] P24-03..P24-04: Delivery confirmation capture (BP6) and returns..."
cargo test -p stheno-payment --test cod_capture_on_delivery
cargo test -p stheno-payment --test cod_return_no_capture

echo "[5/10] P24-05: Settlement report import idempotency (BP7)..."
cargo test -p stheno-payment --test settlement_import

echo "[6/10] P24-06..P24-07: Reconciliation buckets & courier fee balance identity (BP8)..."
cargo test -p stheno-payment --test reconciliation_buckets
cargo test -p stheno-payment --test settlement_balance

echo "[7/10] P24-08..P24-09: Settlement mismatch & aged line discrepancy alerting..."
cargo test -p stheno-payment --test settlement_mismatch
cargo test -p stheno-payment --test settlement_alerting

echo "[8/10] P24-10: Admin reconciliation routes..."
cargo test -p stheno-api-admin --test reconciliation_routes

echo "[9/10] P24-11: Admin reconciliation UI..."
npm --prefix apps/admin run test -- reconciliation

echo "[10/10] P24-12: End-to-end COD lifecycle script..."
./scripts/cod-lifecycle.sh

echo "=========================================================="
echo "STATUS: GATE P24 PASSED (Exit 0)"
echo "=========================================================="
exit 0
