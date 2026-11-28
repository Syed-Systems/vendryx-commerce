#!/usr/bin/env bash
# Phase Gate P26: Branding tier & runtime tokens
set -eo pipefail

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-C:/Users/Mahi/rust-target}"

echo "========================================================"
echo "==> Verifying Phase Gate P26: Branding tier & runtime tokens"
echo "========================================================"

echo ""
echo "[1/10] P26-01: Confirm ADR-012 tenancy audit is green..."
cargo test -p stheno-db --test rls_audit

echo ""
echo "[2/10] P26-02: Tenant branding schema migration & drift check..."
./scripts/check-schema-drift.sh
cargo test -p stheno-db --test rls_audit

echo ""
echo "[3/10] P26-03: Store API branding endpoint & defaults..."
cargo test -p stheno-api-store --test branding_endpoint

echo ""
echo "[4/10] P26-04: Curated self-hosted font licences check..."
./scripts/check-font-licences.sh

echo ""
echo "[5/10] P26-05: Storefront core token resolution & CSS properties..."
npm --prefix apps/storefront run test -- branding-tokens

echo ""
echo "[6/10] P26-06: Logo & favicon type and dimension validation..."
cargo test -p stheno-file --test branding_asset_validation

echo ""
echo "[7/10] P26-07: Admin branding UI with live preview..."
npm --prefix apps/admin run test -- branding

echo ""
echo "[8/10] P26-08: Multi-tenant isolated branding renders..."
./scripts/check-multi-tenant-branding.sh

echo ""
echo "[9/10] P26-08a: Checkout branding & clean script manifest..."
./scripts/check-checkout-branding.sh
./scripts/check-checkout-scripts.sh

echo ""
echo "[10/10] P26-09: Lighthouse audit & 0 round-trip render-path branding..."
npm --prefix apps/storefront run test:lighthouse

echo ""
echo "========================================================"
echo "==> PASS: Phase Gate P26"
echo "========================================================"
