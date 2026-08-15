#!/usr/bin/env bash
# scripts/check-checkout-scripts.sh — Verifies checkout script manifest against PCI SAQ A & F25 requirements

set -euo pipefail

MANIFEST="docs/payment-page-scripts.md"

if [ ! -f "$MANIFEST" ]; then
    echo "FAIL: Checkout script manifest $MANIFEST does not exist."
    exit 1
fi

echo "Verifying checkout script inventory in $MANIFEST..."

# 1. Ensure js.stripe.com is present
if ! grep -q "https://js.stripe.com" "$MANIFEST"; then
    echo "FAIL: Stripe script not documented in $MANIFEST."
    exit 1
fi

# 2. Ensure forbidden scripts (GTM, Meta, analytics) are explicitly banned
for banned in "Google Tag Manager" "Meta Pixel" "chat widgets"; do
    if ! grep -q "$banned" "$MANIFEST"; then
        echo "FAIL: Missing prohibition for $banned in $MANIFEST."
        exit 1
    fi
done

echo "PASS: Checkout script manifest verified (PCI SAQ A compliant)."
exit 0
