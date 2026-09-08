#!/usr/bin/env bash
# scripts/check-brand-surface.sh
# Verifies that public customer-facing surfaces display Vendryx and do not leak
# the legacy customer-facing product name outside reviewed allowlisted identifiers.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Checking customer-facing brand surfaces for legacy public names..."

# Public surfaces to scan
PUBLIC_FILES=(
    "apps/marketing/index.html"
    "apps/admin/index.html"
    "apps/platform/index.html"
    "README.md"
)

# Explicit allowlist patterns (internal crates, compatibility variables, code comments)
# e.g., @stheno/*, STHENO_*, stheno-server, etc.
ALLOWLIST_REGEX='(@stheno/|STHENO_|stheno-|stheno_|X-Stheno-|internal `stheno|engine codename `stheno|`stheno-\*`|codename `stheno`|formerly Stheno|powers Stheno|Stheno Commerce CLI|stheno/|postgres://stheno)'

FAILED=0

for file in "${PUBLIC_FILES[@]}"; do
    if [ ! -f "$file" ]; then
        continue
    fi

    # Check for Stheno or Lokzon as standalone words
    # Strip allowlisted matches
    MATCHES=$(grep -n -i -E '\b(stheno|lokzon)\b' "$file" | grep -v -E "$ALLOWLIST_REGEX" || true)

    if [ -n "$MATCHES" ]; then
        echo "FAIL: Unallowlisted legacy brand match in $file:" >&2
        echo "$MATCHES" >&2
        FAILED=1
    fi
done

# Also check marketing titles and meta tags
MARKETING_APP="apps/marketing/src/App.tsx"
if [ -f "$MARKETING_APP" ]; then
    MATCHES=$(grep -n -i -E '\b(lokzon)\b' "$MARKETING_APP" || true)
    if [ -n "$MATCHES" ]; then
        echo "FAIL: Unallowlisted reference in $MARKETING_APP:" >&2
        echo "$MATCHES" >&2
        FAILED=1
    fi
fi

if [ "$FAILED" -ne 0 ]; then
    echo "FAIL: Brand surface check failed. Update public copy to Vendryx or add internal identifier to allowlist." >&2
    exit 1
fi

echo "Brand surface check PASSED: Customer-facing surfaces are clean."
exit 0
