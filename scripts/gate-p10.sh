#!/usr/bin/env bash
# Phase Gate P10: Storefront, migration, hardening, GA
# Strict build & CI enforcement per AGENTS.md §7
#
# This gate reports the real state of P10. Every check either measures something or
# fails. Nothing here asserts a number it did not observe.

set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P10: Storefront, Migration & Hardening..."
echo "========================================="

echo -e "\n[1/8] Checking Cargo formatting & Clippy across workspace..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/8] Verifying forked storefront builds against the pinned upstream (P10-01, P10-02)..."
if [ ! -f apps/storefront/package.json ]; then
    echo "FAIL: apps/storefront is missing." >&2
    exit 1
fi
npm --prefix apps/storefront ci
npm --prefix apps/storefront run build

echo -e "\n[3/8] Verifying store endpoints the storefront depends on..."
bash scripts/parity.sh --group store-catalog
bash scripts/parity.sh --group store-cart
bash scripts/parity.sh --group store-order
cargo test -p stheno-api-store --test collections_locales_parity

echo -e "\n[4/8] Verifying CSV export carries a UTF-8 BOM for Excel on Windows (P10-03e)..."
cargo test -p stheno-flows --test csv_export_bom

echo -e "\n[5/8] Verifying Medusa import: mapping, money precision, totals, reindex (P10-05 to P10-09)..."
cargo test -p stheno-cli --test import_mapping
cargo test -p stheno-cli --test import_money
cargo test -p stheno-cli --test import_totals_exact
cargo test -p stheno-cli --test import_reindex
cargo test -p stheno-cli --test import_dry_run

echo -e "\n[6/8] Running the encrypted backup restore drill and erasure replay (P10-10)..."
bash scripts/restore-drill.sh

echo -e "\n[7/8] Measuring distroless image size and cold start (P10-11)..."
bash scripts/check-image.sh

echo -e "\n[8/8] Verifying Helm chart, load budgets, PCI SAQ A and disclosure (P10-12 to P10-17)..."
if ! command -v helm >/dev/null 2>&1; then
    echo "FAIL: helm is not installed; the chart is UNVERIFIED (P10-12)." >&2
    exit 1
fi
helm lint deploy/chart
helm template deploy/chart >/dev/null

if ! command -v k6 >/dev/null 2>&1; then
    echo "FAIL: k6 is not installed; the 7 latency budgets are UNVERIFIED (P10-13)." >&2
    exit 1
fi
k6 run tests/load/all.js --quiet

bash scripts/soak.sh --duration 1h --rps 500
bash scripts/check-pci-saq-a.sh
bash scripts/check-disclosure.sh

echo -e "\n--- Docs site (P10-18) ---"
npm --prefix docs-site run build

echo "========================================="
echo "==> Phase Gate P10 PASSED"
echo "========================================="
