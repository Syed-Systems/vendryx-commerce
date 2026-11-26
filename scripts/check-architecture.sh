#!/usr/bin/env bash
# Consolidated architecture checks (replaces 8 individual scripts)
set -euo pipefail
echo "=== Module isolation ==="
./scripts/check-module-isolation.sh
echo "=== SQL boundaries ==="
./scripts/check-sql-boundaries.sh
echo "=== Shared kernel ==="
./scripts/check-shared-kernel.sh
echo "=== Module contracts ==="
./scripts/check-module-contracts.sh
echo "=== DTO separation ==="
./scripts/check-dto-separation.sh
echo "=== No domain gates ==="
./scripts/check-no-domain-gates.sh
echo "=== No in-memory stores ==="
./scripts/check-no-in-memory-stores.sh
echo "=== No enterprise paths ==="
./scripts/check-no-enterprise-paths.sh
echo "=== All architecture checks passed ==="