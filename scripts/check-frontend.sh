#!/usr/bin/env bash
# Consolidated frontend checks (replaces 4 individual scripts)
set -euo pipefail
echo "=== Frontend is not a fork ==="
./scripts/check-frontend-not-forked.sh
echo "=== Frontend binding ==="
./scripts/check-frontend-switch.sh
echo "=== Storefront boundaries ==="
./scripts/check-storefront-boundaries.sh
echo "=== Frontend purity ==="
./scripts/check-frontend-purity.sh
echo "=== Frontend pin ==="
./scripts/check-frontend-pin.sh
echo "=== Frontend resolution ==="
./scripts/check-frontend-resolution.sh
echo "=== All frontend checks passed ==="