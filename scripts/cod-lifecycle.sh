#!/usr/bin/env bash
# scripts/cod-lifecycle.sh — End-to-end COD lifecycle (PHASE3_TASKS.md P24-12)
#
# Order → delivered → captured → settled from courier input alone, decimal-exact:
# 1. COD payment collection authorized
# 2. Courier delivery confirmation authorises capture on COD provider (courier-agnostic, BP6)
# 3. Payout report imported (idempotent, BP7)
# 4. Reconciliation run executed: matched, collected-not-remitted, remitted-not-matched
# 5. Courier fee accounting: sum(remitted) + sum(fees) == sum(collected) in decimal (BP8)
# 6. Deliberate mismatch produces discrepancy row, not silent balance
# 7. Returned COD order produces no capture and no settlement expectation
set -euo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo "=========================================================="
echo "          P24-12: END-TO-END COD LIFECYCLE                "
echo "=========================================================="

echo "[1/4] Running End-to-End COD Lifecycle Suite..."
cargo test -p stheno-payment --test cod_lifecycle

echo "[2/4] Verifying Capture on Delivery (BP6)..."
cargo test -p stheno-payment --test cod_capture_on_delivery

echo "[3/4] Verifying Courier Fee Balance Identity sum(remitted) + sum(fees) == sum(collected) (BP8)..."
cargo test -p stheno-payment --test settlement_balance

echo "[4/4] Verifying Return Handling with Net-Zero Expectation..."
cargo test -p stheno-payment --test cod_return_no_capture

echo "=========================================================="
echo "STATUS: END-TO-END COD LIFECYCLE PASSED (Exit 0)"
echo "=========================================================="
exit 0
