#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking shared kernel (stheno-types)..."

# Ensure stheno-types has no domain models (e.g. Product, Order, Cart)
FORBIDDEN_TYPES="struct Product|struct Order|struct Cart|struct Customer|struct Payment"
MATCHES=$(grep -rnE "$FORBIDDEN_TYPES" crates/core/stheno-types/src/ 2>/dev/null || true)

if [ -n "$MATCHES" ]; then
    echo "ERROR: Domain entities found in stheno-types (AGENTS.md §2.3):"
    echo "$MATCHES"
    exit 1
fi

echo "Shared kernel check PASSED: stheno-types contains primitives only."
exit 0
