#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking SQL boundaries..."

MODULES=$(ls crates/modules)
FAILED=0

for mod in $MODULES; do
    schema_name=$(echo "$mod" | sed 's/stheno-//' | tr '-' '_')
    for other in $MODULES; do
        if [ "$mod" != "$other" ]; then
            other_schema=$(echo "$other" | sed 's/stheno-//' | tr '-' '_')
            # Scan for cross-schema references in sql_query
            MATCHES=$(grep -rn "sql_query.*$other_schema\." "crates/modules/$mod/src" 2>/dev/null || true)
            if [ -n "$MATCHES" ]; then
                echo "ERROR: Cross-schema SQL violation in $mod referencing $other_schema:"
                echo "$MATCHES"
                FAILED=1
            fi
        fi
    done
done

if [ $FAILED -ne 0 ]; then
    echo "SQL boundary check FAILED."
    exit 1
fi

echo "SQL boundary check PASSED: No cross-schema SQL detected."
exit 0
