#!/usr/bin/env bash
# scripts/check-no-enterprise-paths.sh — Verify no enterprise paths remain in dashboard fork (TASKS.md P9-17)
set -euo pipefail

echo "=========================================================="
echo "    CHECK NO ENTERPRISE PATHS AUDIT (P9-17)               "
echo "=========================================================="

echo "[1/2] Scanning dashboard source code for proprietary enterprise paths..."
FORBIDDEN_PATTERNS=(
    "/admin/enterprise"
    "medusa-enterprise"
    "@medusajs/admin-enterprise"
)

for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    if grep -r "$pattern" apps/admin/src/ 2>/dev/null; then
        echo "ERROR: Found forbidden enterprise pattern '$pattern' in apps/admin/src/" >&2
        exit 1
    fi
done
echo "✓ Zero enterprise paths detected in apps/admin/."

echo "[2/2] Verifying clean-room implementations for RBAC and SSO..."
# These were two placeholder files under src/components/ until df42823 replaced them with
# real route trees. The assertion is unchanged — RBAC and SSO must exist here as our own
# implementations rather than as Medusa Enterprise imports, which is what [1/2] enforces —
# only the paths are updated to where those implementations actually live.
CLEAN_ROOM=(
    apps/admin/src/routes/roles
    apps/admin/src/routes/policies
    apps/admin/src/routes/login/components/sso-login.tsx
)
for path in "${CLEAN_ROOM[@]}"; do
    if [ ! -e "$path" ]; then
        echo "ERROR: clean-room implementation missing: $path" >&2
        exit 1
    fi
done
echo "✓ Clean-room RBAC and SSO implementations verified."

echo "=========================================================="
echo "STATUS: NO ENTERPRISE PATHS AUDIT PASSED (Exit 0)"
echo "=========================================================="
exit 0
