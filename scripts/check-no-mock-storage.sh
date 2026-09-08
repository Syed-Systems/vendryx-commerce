#!/usr/bin/env bash
# scripts/check-no-mock-storage.sh
# Enforces ADR-029: No mock storage or simulated persistence in merchant admin production routes.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Checking apps/admin/src/routes for forbidden mock storage..."

ROUTES_DIR="$REPO_ROOT/apps/admin/src/routes"
if [ ! -d "$ROUTES_DIR" ]; then
    echo "Directory $ROUTES_DIR does not exist; skipping check."
    exit 0
fi

FAILED=0

# Find all ts/tsx files excluding __tests__ and test specs
while IFS= read -r file; do
    rel_path="${file#$REPO_ROOT/}"
    if grep -n -E '\b(localStorage|sessionStorage)\b' "$file"; then
        echo "FAIL: Prohibited browser storage in $rel_path"
        FAILED=1
    fi
done < <(find "$ROUTES_DIR" -type f \( -name "*.ts" -o -name "*.tsx" \) ! -path "*/__tests__/*" ! -name "*.spec.*" ! -name "*.test.*")

if [ "$FAILED" -ne 0 ]; then
    echo "FAIL: Found prohibited mock storage in apps/admin routes. Back all state with genuine backend endpoints per ADR-029."
    exit 1
else
    echo "Mock storage check PASSED: apps/admin routes contain zero mock storage."
    exit 0
fi
