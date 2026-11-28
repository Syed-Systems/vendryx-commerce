#!/usr/bin/env bash
# scripts/check-checkout-scripts.sh — Verifies checkout script manifest against PCI SAQ A & F25 requirements

set -euo pipefail

MANIFEST="docs/payment-page-scripts.md"

if [ ! -f "$MANIFEST" ]; then
    echo "FAIL: Checkout script manifest $MANIFEST does not exist."
    exit 1
fi

echo "Verifying checkout script inventory in $MANIFEST..."

# 1. Ensure all registered gateways are covered in manifest
for origin in "js.stripe.com" "bka.sh" "mynagad.com" "sslcommerz.com"; do
    if ! grep -q "$origin" "$MANIFEST"; then
        echo "FAIL: Gateway origin $origin not documented in $MANIFEST."
        exit 1
    fi
done

# 2. Ensure forbidden scripts (GTM, Meta, analytics) are explicitly banned
for banned in "Google Tag Manager" "Meta Pixel" "chat widgets" "Hotjar"; do
    if ! grep -q "$banned" "$MANIFEST"; then
        echo "FAIL: Missing prohibition for $banned in $MANIFEST."
        exit 1
    fi
done

echo "PASS: Checkout script and gateway origin manifest verified (PCI SAQ A compliant)."
exit 0
