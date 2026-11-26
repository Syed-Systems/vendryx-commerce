#!/usr/bin/env bash
# scripts/check-runbooks.sh
# P28-05: Operational incident runbooks verification gate.
#
# Validates that:
# 1. Required runbooks exist in docs/runbooks/.
# 2. Each runbook defines severity, detection, diagnostic workflow, mitigation, and post-incident verification.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "==> Verifying operational incident runbooks (P28-05)..."

RUNBOOKS=(
    "docs/runbooks/reconciliation-discrepancy.md"
    "docs/runbooks/stuck-consignment.md"
    "docs/runbooks/gateway-outage.md"
)

REQUIRED_SECTIONS=(
    "Incident Definition"
    "Detection"
    "Investigation"
    "Mitigation"
    "Post-Incident"
)

for runbook in "${RUNBOOKS[@]}"; do
    if [ ! -f "$runbook" ]; then
        echo "❌ Missing required runbook: $runbook"
        exit 1
    fi
    echo "✓ Found $runbook"

    for section in "${REQUIRED_SECTIONS[@]}"; do
        if ! grep -qi "$section" "$runbook"; then
            echo "❌ Runbook $runbook missing required section: $section"
            exit 1
        fi
    done
    echo "  ✓ All required sections present in $runbook"
done

echo "==> All operational runbooks verified successfully (P28-05 PASSED)."
exit 0
