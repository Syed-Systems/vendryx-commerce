#!/usr/bin/env bash
# scripts/check-multi-tenant-branding.sh
# P26-08: Two tenants, one process, different renders.
#
# Validates that:
# 1. Multi-tenant isolation is enforced at the database/RLS and API layer.
# 2. Storefront token resolution emits distinct CSS custom properties for different tenants.
# 3. No per-tenant rebuild or process restart is required (runtime resolution).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Verifying multi-tenant branding isolation & distinct renders (P26-08)..."

if [ -z "${DATABASE_URL:-}" ]; then
    export DATABASE_URL="postgres://stheno:stheno@localhost:5432/stheno"
fi

echo "1. Checking backend RLS multi-tenant branding isolation..."
cargo test -p stheno-api-store --test branding_endpoint multi_tenant_branding_isolation

echo "2. Checking storefront runtime token resolution & CSS properties..."
npm --prefix apps/storefront run test -- branding-tokens

echo "Multi-tenant branding check PASSED: Distinct renders per tenant verified."
exit 0
