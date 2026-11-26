#!/usr/bin/env bash
# Phase Gate P27: Theme packages & conformance
set -eo pipefail
cd "$(dirname "$0")/.."

echo "=========================================================="
echo "          RUNNING PHASE GATE P27 (THEME CONFORMANCE)       "
echo "=========================================================="

echo ""
echo "[0/10] Storefront structural boundaries..."
./scripts/check-storefront-boundaries.sh
./scripts/check-frontend-purity.sh
./scripts/check-frontend-pin.sh
./scripts/check-frontend-resolution.sh
./scripts/check-frontend-not-forked.sh
./scripts/check-frontend-switch.sh

echo ""
echo "[1/10] P27-01: Theme conformance e2e pack (frontend-default)..."
npm --prefix apps/storefront run test:conformance -- --frontend=default

echo ""
echo "[2/10] P27-02: Conformance gate script..."
./scripts/check-frontend-conformance.sh

echo ""
echo "[3/10] P27-03: Storefront typecheck & contract conformance..."
npm --prefix apps/storefront run typecheck

echo ""
echo "[4/10] P27-04: Theme B catalog and product surfaces..."
npm --prefix apps/storefront run test:conformance -- --frontend=b

echo ""
echo "[5/10] P27-05: Theme B cart and account surfaces..."
npm --prefix apps/storefront run test:conformance -- --frontend=b

echo ""
echo "[6/10] P27-06: Checkout skinning, script manifest & ownership..."
./scripts/check-checkout-scripts.sh
npm --prefix apps/storefront run test -- checkout-ownership

echo ""
echo "[7/10] P27-07: Core delta budget audit..."
test -f docs/FRONTEND_MATRIX_CORE_DELTAS.md
./scripts/check-core-delta-budget.sh

echo ""
echo "[8/10] P27-08: Per-tenant themed build & deploy pipeline..."
./scripts/deploy-themed-storefront.sh --dry-run

echo ""
echo "[9/10] P27-09: Bundler frontend switch verification..."
./scripts/check-frontend-switch.sh

echo ""
echo "[10/10] P27-10: Theme authoring documentation..."
test -f docs-site/docs/frontend-authoring.md
./scripts/check-frontend-docs.sh

echo ""
echo "=========================================================="
echo "STATUS: GATE P27 PASSED (Exit 0)"
echo "=========================================================="
exit 0
