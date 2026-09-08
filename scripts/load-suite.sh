#!/usr/bin/env bash
# scripts/load-suite.sh
# P28-04: Multi-tenant and multi-theme load test runner.
#
# Asserts:
# 1. Multiple tenants and themes running concurrently in one runtime.
# 2. Sub-50ms TTFB latency budgets held under load.
# 3. 0-round-trip SSR branding resolution on the render path (< 0.05ms).
# 4. Core Web Vitals baseline held (Performance score >= 95).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

MULTI_TENANT=false
for arg in "$@"; do
    case "$arg" in
        --multi-tenant)
            MULTI_TENANT=true
            ;;
        *)
            ;;
    esac
done

echo "==> Stheno Load Suite (P28-04)..."

if [ "$MULTI_TENANT" = true ] || [ $# -eq 0 ]; then
    echo "1. Running multi-tenant & multi-theme concurrent load test..."
    node scripts/multi-tenant-load.js

    echo "2. Verifying Core Web Vitals & 0-round-trip SSR branding in headless Chromium..."
    npm --prefix apps/storefront run test:lighthouse
fi

echo "==> Load Suite PASSED: Multi-tenant latency and branding budgets verified."
exit 0
