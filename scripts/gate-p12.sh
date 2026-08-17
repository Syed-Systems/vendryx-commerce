#!/usr/bin/env bash
# Phase Gate P12: Catalog, Pricing, Inventory, Config Persistence.
set -euo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P12: Catalog, Pricing, Inventory, Config Persistence"
echo "========================================================"

if [ -z "${DATABASE_URL:-}" ]; then
    export DATABASE_URL="postgres://stheno:stheno@localhost:5432/stheno"
fi

echo -e "\n[1/7] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/7] P12-01 to P12-05: Pricing persistence & hot path..."
cargo test -p stheno-pricing --test schema
cargo test -p stheno-pricing

echo -e "\n[3/7] P12-06 to P12-08: Inventory persistence, no-oversell & concurrent reservations..."
cargo test -p stheno-inventory --test schema
cargo test -p stheno-inventory --test inventory
cargo test -p stheno-inventory --test reservations
cargo test -p stheno-inventory --test no_oversell
cargo test -p stheno-inventory --test concurrent_reservation

echo -e "\n[4/7] P12-09 to P12-12: Stock Location, Sales Channel, Tax, Store..."
cargo test -p stheno-stock-location
cargo test -p stheno-sales-channel
cargo test -p stheno-tax
cargo test -p stheno-store

echo -e "\n[5/7] P12-13 to P12-14: Product variant database uniqueness..."
cargo test -p stheno-product --test variant_identity
cargo test -p stheno-product --test variant_identity_concurrent

echo -e "\n[6/7] P12-19 to P12-20: Store Parity against Postgres..."
./scripts/parity.sh --group store-catalog
./scripts/parity.sh --group store-pricing

echo -e "\n[7/7] P12-21 to P12-24: Admin Parity against Postgres..."
./scripts/parity.sh --group admin-catalog
./scripts/parity.sh --group admin-inventory
./scripts/parity.sh --group admin-pricing
./scripts/parity.sh --group admin-config

echo "========================================================"
echo "==> Phase Gate P12 PASSED"
echo "========================================================"
