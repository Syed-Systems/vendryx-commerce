#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P2 (Catalog and Inventory)..."
echo "========================================="

echo "[1/6] Checking Architectural Isolation & Contracts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh

echo "[2/6] Running Product & Service Derive Tests..."
cargo test -p stheno-product
cargo test -p stheno-db --test service_derive

echo "[3/6] Running Inventory & Concurrency Oversell Tests (P2-13)..."
cargo test -p stheno-inventory
cargo test -p stheno-stock-location -p stheno-sales-channel
cargo test -p stheno-links --test catalog_links

echo "[4/6] Running Catalog Sagas & Quota Lifecycle Workflows..."
cargo test -p stheno-flows --test product
cargo test -p stheno-flows --test inventory
cargo test -p stheno-flows --test batch_products
cargo test -p stheno-flows --test quota_step
cargo test -p stheno-flows --test quota_concurrent
cargo test -p stheno-flows --test trial_lifecycle
cargo test -p stheno-flows --test grandfathering
cargo test -p stheno-flows --test window_gate

echo "[5/6] Running Upload Security & Egress Guard Gates..."
cargo test -p stheno-file --test upload_security
cargo test -p stheno-http --test egress_guard

echo "[6/6] Running Store Catalog Parity Suite..."
./scripts/parity.sh --group store-catalog

echo "========================================="
echo "==> Phase Gate P2 PASSED: 100% COMPLETE & VERIFIED!"
echo "========================================="
