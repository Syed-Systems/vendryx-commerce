#!/usr/bin/env bash
# scripts/sync-brand-surfaces.sh
# Synchronizes customer-facing brand surfaces with brand/vendryx/brand.json
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

MANIFEST="brand/vendryx/brand.json"
if [ ! -f "$MANIFEST" ]; then
    echo "FAIL: Brand manifest not found at $MANIFEST" >&2
    exit 1
fi

MODE="${1:-}"
if [ "$MODE" != "--write" ] && [ "$MODE" != "--check" ]; then
    echo "Usage: $0 --write | --check" >&2
    exit 1
fi

PRODUCT_NAME=$(grep -oE '"productName": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)
LEGAL_NAME=$(grep -oE '"legalName": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)
DOMAIN=$(grep -oE '"canonicalDomain": "[^"]+"' "$MANIFEST" | head -n 1 | cut -d'"' -f4)

echo "==> Operating on brand: $PRODUCT_NAME ($LEGAL_NAME), domain: $DOMAIN"

TARGETS=(
    "apps/marketing/index.html"
    "apps/admin/index.html"
    "apps/platform/index.html"
)

DRIFT=0

for target in "${TARGETS[@]}"; do
    if [ ! -f "$target" ]; then
        continue
    fi

    DESIRED_TITLE="$PRODUCT_NAME"
    if [[ "$target" == *"platform"* ]]; then
        DESIRED_TITLE="$PRODUCT_NAME Platform"
    elif [[ "$target" == *"admin"* ]]; then
        DESIRED_TITLE="$PRODUCT_NAME Dashboard"
    fi

    CURRENT_TITLE=$(grep -oE '<title>[^<]+</title>' "$target" | sed -E 's/<title>(.*)<\/title>/\1/' || echo "")

    if [ "$CURRENT_TITLE" != "$DESIRED_TITLE" ]; then
        DRIFT=1
        if [ "$MODE" == "--write" ]; then
            sed -i -E "s|<title>[^<]+</title>|<title>$DESIRED_TITLE</title>|g" "$target"
            echo "  Updated title in $target -> $DESIRED_TITLE"
        else
            echo "  Drift in $target: title is '$CURRENT_TITLE', expected '$DESIRED_TITLE'"
        fi
    fi
done

if [ "$MODE" == "--check" ]; then
    if [ "$DRIFT" -ne 0 ]; then
        echo "FAIL: Brand surface drift detected. Run $0 --write to synchronize." >&2
        exit 1
    else
        echo "Brand surfaces check PASSED: All customer-facing targets match $MANIFEST"
        exit 0
    fi
else
    echo "Brand surfaces synchronized successfully."
    exit 0
fi
