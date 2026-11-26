#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Running Phase P1 Gate Suite..."
echo "========================================="

echo "1. Checking formatting..."
cargo fmt --all -- --check

echo "2. Running Clippy..."
cargo clippy --workspace --all-targets -- -D warnings

echo "3. Running all workspace tests..."
cargo test --workspace

echo "4. Running P1 Specific Gates..."
# P1-08: Compensation matrix
cargo test -p stheno-workflow --test compensation_matrix
# P1-14: Kill and resume harness
./scripts/kill-resume.sh
# P1-15: Idempotency concurrent
cargo test -p stheno-workflow --test idempotency_concurrent
# P1-20: Zero N+1 queries
cargo test -p stheno-query --test no_n_plus_one
# P1-23: Order change corpus
cargo test -p stheno-order-change --test corpus
# P1-24: Composition root
cargo run -p stheno-server -- --check-config

echo "========================================="
echo "==> Phase P1 Gate PASSED: 100% GREEN!"
echo "========================================="
exit 0
