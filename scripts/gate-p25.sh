#!/usr/bin/env bash
# Phase Gate P25: Storefront core/theme split
set -eo pipefail

APP="apps/storefront"

echo "========================================================"
echo "==> Verifying Phase Gate P25: Storefront core/theme split"
echo "========================================================"

echo ""
echo "[1/8] Lint, typecheck, unit tests..."
npm --prefix "$APP" run lint
npm --prefix "$APP" run typecheck
npm --prefix "$APP" run test

echo ""
echo "[2/8] P25-01/04: the app shell builds from core + theme..."
npm --prefix "$APP" run build

echo ""
echo "[3/8] P25-02/11: contract inputs and versioning policy are recorded..."
test -f docs/THEME_CONTRACT_INPUTS.md
test -f docs/THEME_VERSIONING.md
./scripts/check-theme-pin.sh

echo ""
echo "[4/8] P25-07: core binds a theme in exactly one place..."
./scripts/check-theme-resolution.sh

echo ""
echo "[5/8] P25-09: a theme cannot reach into core..."
./scripts/check-storefront-boundaries.sh

echo ""
echo "[6/8] P25-10: a theme presents and nothing else..."
./scripts/check-theme-purity.sh

echo ""
echo "[7/8] P25-08: checkout scripts still satisfy SAQ-A..."
./scripts/check-checkout-scripts.sh

echo ""
echo "[8/8] P25-13: the extraction changed no rendered output..."
./scripts/check-extraction-parity.sh

echo ""
echo "========================================================"
echo "==> PASS: Phase Gate P25"
echo "========================================================"
