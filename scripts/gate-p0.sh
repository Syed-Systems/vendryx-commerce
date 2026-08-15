#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Running Phase P0 Gate Suite..."
echo "========================================="

echo "1. Checking formatting..."
cargo fmt --all -- --check

echo "2. Running Clippy..."
cargo clippy --workspace --all-targets -- -D warnings

echo "3. Running all workspace tests..."
cargo test --workspace

echo "4. Running Boundary Scripts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-dto-separation.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh
./scripts/check-migration-phases.sh
./scripts/check-schema-drift.sh

echo "5. Running Negative Boundary Tests (P0-25)..."
./scripts/test-boundary-checks.sh

echo "========================================="
echo "==> Phase P0 Gate PASSED: 100% GREEN!"
echo "========================================="
exit 0
