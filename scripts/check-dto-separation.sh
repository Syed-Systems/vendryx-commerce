#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking DTO / Entity separation..."

# Ensure no entity models derive Deserialize
VIOLATIONS=$(grep -rn "derive.*Deserialize" crates/modules/*/src/models.rs 2>/dev/null || true)

if [ -n "$VIOLATIONS" ]; then
    echo "ERROR: Entity models must never derive Deserialize (AGENTS.md §2.6):"
    echo "$VIOLATIONS"
    exit 1
fi

echo "DTO separation check PASSED: No entity models derive Deserialize."
exit 0
