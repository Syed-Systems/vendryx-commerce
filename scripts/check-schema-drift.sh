#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking schema drift..."

# Check git status for modified schema.rs files
if git status --porcelain | grep -E "schema\.rs" > /dev/null 2>&1; then
    echo "ERROR: Uncommitted schema.rs changes detected."
    git diff -- "crates/modules/*/src/schema.rs"
    exit 1
fi

echo "Schema drift check PASSED: No schema drift detected."
exit 0
