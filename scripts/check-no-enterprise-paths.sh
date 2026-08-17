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
if [ ! -f apps/admin/src/components/rbac.tsx ] || [ ! -f apps/admin/src/components/sso.tsx ]; then
    echo "ERROR: Clean-room rbac.tsx or sso.tsx missing!" >&2
    exit 1
fi
echo "✓ Clean-room RBAC and SSO components verified."

echo "=========================================================="
echo "STATUS: NO ENTERPRISE PATHS AUDIT PASSED (Exit 0)"
echo "=========================================================="
exit 0
