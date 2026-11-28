#!/usr/bin/env bash
# Phase Gate P20: Hardening, Load, and GA Verification
set -eo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo "========================================================"
echo "==> Verifying Phase Gate P20: Hardening, Load, and GA"
echo "========================================================"

echo ""
echo "[1/5] P20-18: Security disclosure & SBOM compliance audit..."
./scripts/check-disclosure.sh

echo ""
echo "[2/5] P20-17: PCI DSS SAQ A and checkout script integrity..."
./scripts/check-pci-saq-a.sh

echo ""
echo "[3/5] P20-15: Redis outage and degradation recovery verification..."
./scripts/redis-outage.sh

echo ""
echo "[4/5] Full Parity test suite verification..."
./scripts/parity.sh

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
echo "==> Phase Gate P20 PASSED: GA Production Ready"
echo "========================================================"
