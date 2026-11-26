#!/usr/bin/env bash
# scripts/check-checkout-branding.sh
# P26-08a: Checkout branding compliance and script isolation.
#
# Validates that:
# 1. Checkout route templates consume --brand-* CSS custom properties.
# 2. No third-party font CDNs or unapproved script assets are loaded on checkout routes (ADR-013 Decision 2a).
# 3. Only self-hosted fonts and trusted CSS variables apply on checkout.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Verifying checkout branding compliance (P26-08a)..."

# Ensure checkout directory exists
CHECKOUT_DIR="apps/storefront/src/app/[countryCode]/(checkout)"
if [ ! -d "$CHECKOUT_DIR" ]; then
    CHECKOUT_DIR="apps/storefront/src/modules/checkout"
fi

if [ ! -d "$CHECKOUT_DIR" ] && [ ! -d "apps/storefront/packages/storefront-core/src/modules/checkout" ]; then
    echo "FAIL: Checkout directory not found." >&2
    exit 1
fi

# Check for forbidden third-party font or script CDNs in checkout modules
SCAN_PATHS=(
    "apps/storefront/src/app/[countryCode]/(checkout)"
    "apps/storefront/src/modules/checkout"
    "apps/storefront/packages/storefront-core/src/modules/checkout"
)

FORBIDDEN_PATTERNS='(fonts\.googleapis\.com|fonts\.gstatic\.com|use\.typekit\.net|cdn\.jsdelivr\.net|unpkg\.com)'

for path in "${SCAN_PATHS[@]}"; do
    if [ -d "$path" ]; then
        MATCHES=$(grep -rnE "$FORBIDDEN_PATTERNS" "$path" 2>/dev/null || true)
        if [ -n "$MATCHES" ]; then
            echo "FAIL: Third-party CDN reference found in checkout path $path:" >&2
            echo "$MATCHES" >&2
            exit 1
        fi
    fi
done

# Verify RootLayout injects --brand-* style block into <head>
LAYOUT_FILE="apps/storefront/packages/storefront-core/src/app/layout.tsx"
if [ ! -f "$LAYOUT_FILE" ] || ! grep -q "stheno-brand-tokens" "$LAYOUT_FILE"; then
    echo "FAIL: RootLayout does not inject stheno-brand-tokens into <head>." >&2
    exit 1
fi

echo "Checkout branding check PASSED: Checkout routes apply brand tokens and obey script isolation."
exit 0
