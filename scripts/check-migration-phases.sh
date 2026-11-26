#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking migration phases (AGENTS.md §2.13)..."

# Phased migration check: destructive statements must have '-- phase: contract'
MIGRATION_FILES=$(find crates/modules -name "*.sql" 2>/dev/null || true)
FAILED=0

for file in $MIGRATION_FILES; do
    # Check for DROP COLUMN, ALTER COLUMN TYPE without phase marker
    if grep -iE "DROP\s+COLUMN|ALTER\s+COLUMN.*TYPE" "$file" > /dev/null 2>&1; then
        if ! grep -q "\-\- phase: contract" "$file"; then
            echo "ERROR: Destructive statement in $file without '-- phase: contract' marker:"
            grep -inE "DROP\s+COLUMN|ALTER\s+COLUMN.*TYPE" "$file"
            FAILED=1
        fi
    fi
done

if [ $FAILED -ne 0 ]; then
    echo "Migration phases check FAILED."
    exit 1
fi

echo "Migration phases check PASSED: All destructive statements are phased."
exit 0
