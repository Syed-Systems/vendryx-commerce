#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P3 (Pricing, Tax, Region, Currency)..."
echo "========================================="

echo "[1/7] Checking Architectural Isolation & Contracts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh

echo "[2/7] Running Currency & Precision Table Tests (P3-01)..."
cargo test -p stheno-currency

echo "[3/7] Running Region & Store Tests (P3-02, P3-03)..."
cargo test -p stheno-region
cargo test -p stheno-store

echo "[4/7] Running Pricing Engine & Specificity Tests (P3-04 to P3-11)..."
cargo test -p stheno-pricing

echo "[5/7] Running Tax Engine, Exact Reverse & Provider Tests (P3-12 to P3-15)..."
cargo test -p stheno-tax

echo "[6/7] Running Pricing Links & Sagas (P3-16, P3-17)..."
cargo test -p stheno-links --test pricing_links
cargo test -p stheno-flows --test pricing --test tax --test region --test store

echo "[7/7] Running Store Pricing Parity Suite (P3-18)..."
./scripts/parity.sh --group store-pricing

echo "========================================="
echo "==> Phase Gate P3 PASSED: 100% COMPLETE & VERIFIED!"
echo "========================================="
exit 0
