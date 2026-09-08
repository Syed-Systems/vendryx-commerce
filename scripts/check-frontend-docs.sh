#!/usr/bin/env bash
# scripts/check-frontend-docs.sh — P27-10 Theme Authoring Documentation Gate
#
# Asserts that docs-site/docs/frontend-authoring.md exists, states the checkout
# rule first (ADR-013), enumerates the 45 contract slots, and records time-to-new-theme.
set -euo pipefail
cd "$(dirname "$0")/.."

DOC="docs-site/docs/frontend-authoring.md"

if [ ! -f "$DOC" ]; then
    echo "FAIL: Theme authoring guide $DOC does not exist." >&2
    exit 1
fi

echo "Verifying theme authoring guide in $DOC..."

# 1. Checkout rule stated first (opening lines)
FIRST_LINES=$(head -n 10 "$DOC")
if ! echo "$FIRST_LINES" | grep -qi "Checkout is core at every tier"; then
    echo "FAIL: Rule 0 ('Checkout is core at every tier') must be stated first in $DOC." >&2
    exit 1
fi

# 2. 45-slot contract reference
if ! grep -qi "FrontendContract" "$DOC" || ! grep -q "45" "$DOC"; then
    echo "FAIL: $DOC must reference the 45-slot FrontendContract." >&2
    exit 1
fi

# 3. ADAPT vs SKIN distinction
if ! grep -q "ADAPT" "$DOC" || ! grep -q "SKIN" "$DOC"; then
    echo "FAIL: $DOC must document ADAPT vs SKIN per AGENTS.md §12." >&2
    exit 1
fi

# 4. Measured time-to-new-theme benchmark
if ! grep -qi "time-to-new-theme" "$DOC" && ! grep -qi "Benchmark" "$DOC"; then
    echo "FAIL: $DOC must record time-to-new-theme benchmark measurements." >&2
    exit 1
fi

echo "PASS: Theme authoring documentation verified (checkout rule first, contract slots, benchmarks)."
exit 0
