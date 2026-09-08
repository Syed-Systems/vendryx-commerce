#!/usr/bin/env bash
# Phase Gate P28: Hardening, PCI re-attestation, GA
set -eo pipefail
cd "$(dirname "$0")/.."

echo "=========================================================="
echo "          RUNNING PHASE GATE P28 (HARDENING & GA)         "
echo "=========================================================="

if [ -z "${DATABASE_URL:-}" ]; then
    export DATABASE_URL="postgres://stheno:stheno@localhost:5432/stheno"
fi

echo ""
echo "[0/5] Universal structural & architecture boundary checks..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-storefront-boundaries.sh
./scripts/check-frontend-purity.sh
./scripts/check-frontend-pin.sh
./scripts/check-frontend-resolution.sh
./scripts/check-frontend-not-forked.sh

echo ""
echo "[1/5] P28-01: Gateway redirect & iframe CSP script manifest..."
./scripts/check-checkout-scripts.sh

echo ""
echo "[2/5] P28-02: PCI DSS v4.0 SAQ-A re-attestation across all providers..."
./scripts/check-pci-saq-a.sh

echo ""
echo "[3/5] P28-03: Provider failure modes (timeout, 500, callback loss, courier down)..."
cargo test --workspace --test provider_failure_modes

echo ""
echo "[4/5] P28-04: Multi-tenant & multi-theme load suite (budgets on render path)..."
./scripts/load-suite.sh --multi-tenant

echo ""
echo "[5/5] P28-05: Operational incident runbooks verification..."
./scripts/check-runbooks.sh

echo ""
echo "=========================================================="
echo "  ALL PHASE 28 GATES PASSED (100% PHASE 3 GA VERIFIED)    "
echo "=========================================================="
exit 0
