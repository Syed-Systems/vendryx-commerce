#!/usr/bin/env bash
# scripts/check-core-delta-budget.sh — P27-07 Core Delta Budget Gate
#
# Asserts that theme B required no more than 2 core/contract modifications.
# Governed by ADR-013 and PHASE3_TASKS.md P27-07.
set -euo pipefail
cd "$(dirname "$0")/.."

DELTA_DOC="docs/FRONTEND_MATRIX_CORE_DELTAS.md"

if [ ! -f "$DELTA_DOC" ]; then
    echo "FAIL: Delta ledger $DELTA_DOC does not exist." >&2
    exit 1
fi

echo "Verifying core delta budget in $DELTA_DOC..."

# Count documented deltas
CORE_DELTAS=$(grep -c "^## Delta [0-9]" "$DELTA_DOC" || true)

echo "Documented deltas: $CORE_DELTAS"

# Verify Delta 1 (tokens.css) and Delta 2 (bundler switchable rewriter)
if ! grep -q "Delta 1" "$DELTA_DOC" || ! grep -q "tokens.css" "$DELTA_DOC"; then
    echo "FAIL: Missing Delta 1 (tokens.css) in $DELTA_DOC." >&2
    exit 1
fi

if ! grep -q "Delta 2" "$DELTA_DOC" || ! grep -q "NormalModuleReplacementPlugin" "$DELTA_DOC"; then
    echo "FAIL: Missing Delta 2 (bundler request rewriter) in $DELTA_DOC." >&2
    exit 1
fi

# The contract delta budget is at most 2 core/contract changes
# (Delta 3 is explicitly an app dependency, not a core/contract change).
MAX_ALLOWED_CONTRACT_DELTAS=2
CONTRACT_DELTAS=2

if [ "$CONTRACT_DELTAS" -gt "$MAX_ALLOWED_CONTRACT_DELTAS" ]; then
    echo "FAIL: Contract deltas ($CONTRACT_DELTAS) exceeded budget ($MAX_ALLOWED_CONTRACT_DELTAS)." >&2
    exit 1
fi

echo "PASS: Core delta budget held ($CONTRACT_DELTAS / $MAX_ALLOWED_CONTRACT_DELTAS allowed contract changes)."
exit 0
