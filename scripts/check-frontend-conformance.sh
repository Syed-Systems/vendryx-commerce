#!/usr/bin/env bash
# scripts/check-frontend-conformance.sh — P27-02 Theme Conformance Gate
#
# Verifies that frontend-default (and bound themes) conform strictly to
# the FrontendContract (45 slots) and pass all 5 customer journey stages.
#
# Governed by ADR-013 decision 9 and AGENTS.md §12.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Running theme conformance gate against frontend-default..."
npm --prefix apps/storefront run test:conformance -- --frontend=default

echo "==> Conformance gate passed for frontend-default."
exit 0
