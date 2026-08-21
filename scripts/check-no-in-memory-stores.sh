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

# P37-01: the API layer, too.
#
# This check only ever looked at crates/modules, so the control plane accumulated nine
# process-global registries in crates/api without tripping it -- impersonation sessions,
# an incident read-only lock, a gateway kill switch, theme assignments, feature flags.
# Every one of them dies on restart and was never shared between nodes.
#
# The list below is a shrinking ledger, not an exemption. Three things fail:
#   - a violation in a file that is not listed        (no new ones)
#   - more violations in a listed file than recorded  (no growth)
#   - a listed file with none left                    (delete the stale entry)
# The last rule is what stops the ledger outliving the debt. Each P37 class-B task
# removes one row; when the list is empty the `[ ${#API_LEDGER[@]} -eq 0 ]` case below
# is the permanent state and the API layer is held to the same rule as the modules.
API_PATTERN='OnceLock<[[:space:]]*(RwLock|Mutex)<'
API_LEDGER=(
)

echo "==> Checking for process-global state in crates/api..."

declare -A API_ALLOWED=()
for entry in "${API_LEDGER[@]}"; do
    API_ALLOWED["${entry% *}"]="${entry##* }"
done

# Comment lines are excluded the same way the module pass excludes them. Without this a
# doc comment explaining what a registry used to be counts as one, so removing a registry
# and documenting why fails the gate — which would teach exactly the wrong lesson.
api_hits() {
    grep -nE "$API_PATTERN" "$1" 2>/dev/null \
        | grep -vE '^[0-9]+:[[:space:]]*(//|/\*|\*)' || true
}

while IFS= read -r file; do
    count=$(api_hits "$file" | grep -c . || true)
    allowed="${API_ALLOWED[$file]:-0}"
    if [ "$count" -gt "$allowed" ]; then
        echo "FAIL: ${file} holds ${count} process-global registr(ies), ledger allows ${allowed}:" >&2
        api_hits "$file" >&2
        echo "      Control-plane state belongs in Postgres. See docs/CONTROL_PLANE_PLAN.md class B." >&2
        FAILED=1
    fi
done < <(grep -rlE "$API_PATTERN" --include='*.rs' crates/api/ | grep -v '/tests/' | sort)

for file in "${!API_ALLOWED[@]}"; do
    if [ ! -f "$file" ] || [ "$(api_hits "$file" | grep -c . || true)" -eq 0 ]; then
        echo "FAIL: ${file} is on the in-memory ledger but holds none. Delete the entry." >&2
        FAILED=1
    fi
done

if [ "$FAILED" -ne 0 ]; then
    echo "In-memory store check FAILED." >&2
    exit 1
fi

REMAINING=0
for entry in "${API_LEDGER[@]}"; do REMAINING=$((REMAINING + ${entry##* })); done
echo "In-memory store check PASSED: ${CHECKED} migrated module(s) hold no in-process state."
if [ "$REMAINING" -gt 0 ]; then
    echo "  ledger: ${REMAINING} process-global registr(ies) remain in crates/api, none new."
fi
exit 0
