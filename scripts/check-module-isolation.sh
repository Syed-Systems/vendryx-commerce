#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking module isolation..."

# List of modules
MODULES=$(ls -d crates/modules/*/ | xargs -n 1 basename)

FAILED=0
for mod in $MODULES; do
    CARGO_TOML="crates/modules/$mod/Cargo.toml"
    if [ -f "$CARGO_TOML" ]; then
        for other in $MODULES; do
            if [ "$mod" != "$other" ]; then
                if grep -E "^[[:space:]]*${other}[[:space:]]*=" "$CARGO_TOML" > /dev/null 2>&1; then
                    echo "ERROR: Module isolation violation! $mod depends on $other in $CARGO_TOML"
                    FAILED=1
                fi
            fi
        done
    fi
done

if [ $FAILED -ne 0 ]; then
    echo "Module isolation check FAILED."
    exit 1
fi

echo "Module isolation check PASSED: No cross-module dependencies detected."
exit 0
