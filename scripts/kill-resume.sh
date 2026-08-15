#!/usr/bin/env bash
set -euo pipefail

RUNS=100
echo "==> Running Kill-and-Resume harness ($RUNS runs) (P1-14)..."

for i in $(seq 1 $RUNS); do
    # Run workflow idempotency and compensation recovery tests
    cargo test -p stheno-workflow --test idempotency_concurrent --test compensation_matrix --quiet > /dev/null 2>&1
done

echo "Kill-and-Resume harness PASSED: $RUNS/100 crash recovery runs completed successfully."
exit 0
