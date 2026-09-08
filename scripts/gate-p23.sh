#!/usr/bin/env bash
# scripts/gate-p23.sh — Phase gate for P23: Courier providers and tracking (PHASE3_TASKS.md P23)
set -euo pipefail

echo "=========================================================="
echo "          P23 PHASE GATE: COURIER PROVIDERS & TRACKING    "
echo "=========================================================="

echo "[1/7] Universal gates: Module isolation, boundaries, and kernel purity..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-schema-drift.sh
python scripts/check-no-fabricated-handlers.py

echo "[2/7] P23-01: Courier status vocabulary..."
cargo test -p stheno-fulfillment --test status_vocabulary

echo "[3/7] P23-02..P23-05: Pathao courier provider..."
cargo test -p stheno-fulfillment-pathao --test options
cargo test -p stheno-fulfillment-pathao --test rates
cargo test -p stheno-fulfillment-pathao --test consignment
cargo test -p stheno-fulfillment-pathao --test status_mapping

echo "[4/7] P23-06..P23-07: Steadfast courier provider..."
cargo test -p stheno-fulfillment-steadfast --test provider
cargo test -p stheno-fulfillment-steadfast --test status_mapping

echo "[5/7] P23-08..P23-09: RedX courier provider..."
cargo test -p stheno-fulfillment-redx --test provider
cargo test -p stheno-fulfillment-redx --test status_mapping

echo "[6/7] P23-10..P23-12: Polling fallback, unknown status alerts, zone options (requires DATABASE_URL)..."
cargo test -p stheno-fulfillment --test status_polling
cargo test -p stheno-fulfillment --test unknown_status_alerts
cargo test -p stheno-fulfillment --test zone_options

echo "[7/7] P23-13: Recorded courier wiremock fixtures (no live API calls)..."
./scripts/check-courier-fixtures.sh

echo "=========================================================="
echo "STATUS: GATE P23 PASSED (Exit 0)"
echo "=========================================================="
exit 0
