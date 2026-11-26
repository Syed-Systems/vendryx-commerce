#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking for domain feature gating..."

MATCHES=$(grep -rn "Entitlements" crates/modules/ 2>/dev/null || true)

if [ -n "$MATCHES" ]; then
    echo "ERROR: Entitlements referenced inside crates/modules/ (AGENTS.md §2.4):"
    echo "$MATCHES"
    exit 1
fi

echo "Domain feature gating check PASSED: No Entitlements references in domain modules."
exit 0
