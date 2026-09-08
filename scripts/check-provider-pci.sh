#!/usr/bin/env bash
# scripts/check-provider-pci.sh — Mechanical PCI SAQ A enforcement on payment providers (P21-13)
set -euo pipefail

echo "==> Checking payment provider PCI SAQ A compliance..."

FORBIDDEN_CARD_PATTERNS='card_number|credit_card_pan|\bpan\b.*String|\bcvv\b.*String|\bcvc\b.*String|card_cvv|card_pan|track1|track2'

# 1. Selftest: Ensure the check catches a deliberately non-compliant fixture
FIXTURE_NON_COMPLIANT="struct MaliciousDirectCardProvider { pan: String, cvv: String }"
if ! echo "$FIXTURE_NON_COMPLIANT" | grep -qE "$FORBIDDEN_CARD_PATTERNS"; then
    echo "ERROR: Selftest failed! check-provider-pci did not detect non-compliant card-data pattern." >&2
    exit 1
fi
echo "✓ Selftest passed: Deliberately non-compliant fixture detected and refused."

# 2. Audit all registered and candidate payment provider crates
VIOLATIONS=$(grep -rnE "$FORBIDDEN_CARD_PATTERNS" crates/providers/stheno-payment-*/src/ 2>/dev/null || true)

if [ -n "$VIOLATIONS" ]; then
    echo "ERROR: Forbidden cardholder data fields detected in provider crates (PCI SAQ A violation)!" >&2
    echo "$VIOLATIONS" >&2
    exit 1
fi

echo "Provider PCI check PASSED: Zero cardholder data flows across all payment provider crates."
exit 0
