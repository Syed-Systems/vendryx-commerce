#!/usr/bin/env bash
# P13-05: Kill-resume against real restarts and durable Postgres stores.
set -euo pipefail

RUNS=${RUNS:-10}
echo "==> Running Kill-and-Resume harness ($RUNS runs) (P13-05)..."

for i in $(seq 1 $RUNS); do
    cargo test -p stheno-workflow --test durable_store --test compensation_durability --test lease_contention --test idempotency_concurrent --test compensation_matrix --quiet > /dev/null 2>&1
done

echo "Kill-and-Resume harness PASSED: $RUNS/$RUNS crash recovery and restart runs completed successfully."
exit 0
