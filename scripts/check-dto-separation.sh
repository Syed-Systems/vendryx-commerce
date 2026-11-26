#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking DTO / Entity separation..."

# 1. Ensure models are strictly internal (never pub use re-exported in dto.rs or lib.rs)
REEXPORTS=$(grep -rn "pub use.*models" crates/modules/*/src/ 2>/dev/null || true)
if [ -n "$REEXPORTS" ]; then
    echo "ERROR: Entity models must never be re-exported across the module boundary (AGENTS.md §3.4):"
    echo "$REEXPORTS"
    exit 1
fi

# 2. Ensure API handlers in crates/api do not take internal models directly
API_LEAKS=$(grep -rn "Json<.*models::" crates/api/*/src/ 2>/dev/null || true)
if [ -n "$API_LEAKS" ]; then
    echo "ERROR: API handlers must use Request DTOs, never entity models (AGENTS.md §2.9):"
    echo "$API_LEAKS"
    exit 1
fi

echo "DTO separation check PASSED: Entity models strictly isolated from DTO boundary."
exit 0
