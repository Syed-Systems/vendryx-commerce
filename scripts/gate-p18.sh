#!/usr/bin/env bash
# Phase Gate P18: Admin Dashboard Fork & Verification
set -eo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo "========================================================"
echo "==> Verifying Phase Gate P18: Admin Dashboard Fork"
echo "========================================================"

echo ""
echo "[1/4] P18-02: Checking for zero enterprise paths..."
./scripts/check-no-enterprise-paths.sh

echo ""
echo "[2/4] P18-03 to P18-27: Admin dashboard E2E test suite..."
npm --prefix apps/admin run test:e2e

echo ""
echo "[3/4] P18-28: Admin dashboard WCAG 2.2 AA accessibility audit..."
npm --prefix apps/admin run test:a11y

echo ""
echo "[4/4] Architectural gate checks..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-module-contracts.sh
./scripts/check-no-domain-gates.sh
./scripts/check-no-in-memory-stores.sh
./scripts/check-schema-drift.sh

echo ""
echo "========================================================"
echo "==> Phase Gate P18 PASSED"
echo "========================================================"
