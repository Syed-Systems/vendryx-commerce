#!/usr/bin/env bash
# P11-21, P14-22: a module declared migrated may not hold in-memory state.
set -euo pipefail

# Modules whose services are on Postgres.
MIGRATED=(
    "stheno-region"
    "stheno-currency"
    "stheno-product"
    "stheno-pricing"
    "stheno-inventory"
    "stheno-stock-location"
    "stheno-sales-channel"
    "stheno-tax"
    "stheno-store"
    "stheno-customer"
    "stheno-auth"
    "stheno-user"
    "stheno-api-key"
    "stheno-rbac"
    "stheno-file"
    "stheno-notification"
    "stheno-search"
    "stheno-index"
    "stheno-translation"
    "stheno-settings"
    "stheno-audit"
    "stheno-analytics"
)

# Patterns that constitute in-process storage of domain state.
PATTERN='RwLock<[[:space:]]*HashMap|RwLock<[[:space:]]*Vec|Mutex<[[:space:]]*HashMap|Mutex<[[:space:]]*Vec'

ALL=0
[ "${1:-}" = "--all" ] && ALL=1

if [ "$ALL" -eq 1 ]; then
    mapfile -t MIGRATED < <(ls crates/modules | grep -vE '^(stheno-caching|stheno-locking|stheno-demo)$')
fi

echo "==> Checking for in-memory stores in migrated modules..."

FAILED=0
CHECKED=0

for module in "${MIGRATED[@]}"; do
    src="crates/modules/${module}/src"
    if [ ! -d "$src" ]; then
        echo "FAIL: ${module} is declared migrated but has no source at ${src}" >&2
        FAILED=1
        continue
    fi

    CHECKED=$((CHECKED + 1))
    if MATCHES=$(grep -rnE "$PATTERN" "$src" 2>/dev/null | grep -vE '^[^:]+:[0-9]+:[[:space:]]*(//|/\*|\*)'); then
        echo "FAIL: ${module} is declared migrated but still holds in-memory state:" >&2
        echo "$MATCHES" >&2
        FAILED=1
    fi
done

if [ "$CHECKED" -eq 0 ]; then
    echo "FAIL: no modules were checked; the migrated list is empty." >&2
    exit 1
fi

if [ "$FAILED" -ne 0 ]; then
    echo "In-memory store check FAILED." >&2
    exit 1
fi

echo "In-memory store check PASSED: ${CHECKED} migrated module(s) hold no in-process state."
exit 0
