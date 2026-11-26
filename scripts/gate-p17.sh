#!/usr/bin/env bash
# Phase Gate P17: Admin API Completion & Full Parity Suite
set -eo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P17: Admin API Completion"
echo "========================================================"

echo ""
echo "[1/5] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo ""
echo "[2/5] P17-01 to P17-30: Parity test suite (store and admin)..."
./scripts/parity.sh

echo ""
echo "[3/5] P17-02: Admin route coverage test..."
cargo test -p stheno-api-admin --test route_coverage

echo ""
echo "[4/5] P17-31: Router tier matrix and conditional mounting..."
cargo test -p stheno-api-router --test tier_matrix

echo ""
echo "[5/5] Architectural gate checks..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-schema-drift.sh

echo ""
echo "========================================================"
echo "==> Phase Gate P17 PASSED"
echo "========================================================"
