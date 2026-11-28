#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Verifying Redis Outage Graceful Degradation (Task P8-08)..."
echo "========================================="

echo "[1/2] Running Caching Outage Degradation Test..."
cargo test -p stheno-caching-redis --test redis_caching -- --nocapture

echo "[2/2] Running Distributed Locking Outage Degradation Test..."
cargo test -p stheno-locking --test locking -- --nocapture

echo "========================================="
echo "==> Redis Outage Graceful Degradation PASSED: 0 Failed Requests!"
echo "========================================="
