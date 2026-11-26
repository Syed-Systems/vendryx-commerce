#!/usr/bin/env bash
# Phase Gate P19: Storefront Completion & E2E Verification
set -eo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo "========================================================"
echo "==> Verifying Phase Gate P19: Storefront Completion"
echo "========================================================"

echo ""
echo "[1/3] P19-02 to P19-12, P19-14: Storefront E2E test suite..."
npm --prefix apps/storefront run test:e2e

echo ""
echo "[2/3] P19-13: Storefront Lighthouse and Core Web Vitals audit..."
npm --prefix apps/storefront run test:lighthouse

echo ""
echo "[3/3] Architectural gate checks..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-schema-drift.sh

echo ""
echo "========================================================"
echo "==> Phase Gate P19 PASSED"
echo "========================================================"
