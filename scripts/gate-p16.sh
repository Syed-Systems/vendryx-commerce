#!/usr/bin/env bash
# Phase Gate P16: Reporting Module & Analytics Rollups
set -eo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P16: Reporting Module"
echo "========================================================"

echo ""
echo "[1/6] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo ""
echo "[2/6] P16-01: stheno-reporting unit tests..."
cargo test -p stheno-reporting

echo ""
echo "[3/6] P16-02 to P16-11: Rollup schemas, refresh, invalidation, aggregates, tier horizons..."
cargo test -p stheno-reporting --test rollup_schema
cargo test -p stheno-reporting --test rollup_incremental
cargo test -p stheno-reporting --test rollup_invalidation
cargo test -p stheno-reporting --test revenue_over_time
cargo test -p stheno-reporting --test orders_over_time
cargo test -p stheno-reporting --test top_products
cargo test -p stheno-reporting --test average_order_value
cargo test -p stheno-reporting --test window_gate
cargo test -p stheno-reporting --test free_tier_horizon
cargo test -p stheno-reporting --test pro_tier_horizon

echo ""
echo "[4/6] P16-12: Medusa envelope shape for reports..."
cargo test -p stheno-api-admin --test reports_parity

echo ""
echo "[5/6] P16-13: Report CSV export with UTF-8 BOM..."
cargo test -p stheno-reporting --test report_csv_export

echo ""
echo "[6/6] Architectural gate checks..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-schema-drift.sh

echo ""
echo "========================================================"
echo "==> Phase Gate P16 PASSED"
echo "========================================================"
