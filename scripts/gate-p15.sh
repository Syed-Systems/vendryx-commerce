#!/usr/bin/env bash
# Phase Gate P15: Quota counters and entitlement enforcement (Bash).
set -euo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P15: Quota Counters & Entitlements"
echo "========================================================"

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo -e "\n[1/6] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/6] P15-01 to P15-03: Entitlement usage table, atomic increment, decrement..."
cargo test -p stheno-db --test entitlement_usage_schema
cargo test -p stheno-db --test quota_increment
cargo test -p stheno-db --test quota_decrement

echo -e "\n[3/6] P15-04 to P15-06: CheckQuotaStep, product quota, seat quota..."
cargo test -p stheno-flows --test check_quota_step
cargo test -p stheno-flows --test product_quota
cargo test -p stheno-flows --test seat_quota

echo -e "\n[4/6] P15-07 to P15-08: Exact quota boundary and 100-parallel concurrency..."
cargo test -p stheno-flows --test quota_boundary
cargo test -p stheno-flows --test quota_concurrency

echo -e "\n[5/6] P15-09 to P15-11: Over-quota error, grandfathering, and real entitlements usage..."
cargo test -p stheno-api-admin --test over_quota_error
cargo test -p stheno-flows --test grandfathering
cargo test -p stheno-api-admin-platform --test entitlements

echo -e "\n[6/6] P15-12: Quota reconciliation..."
cargo test -p stheno-flows --test quota_reconciliation

echo "========================================================"
echo "==> Phase Gate P15 PASSED"
echo "========================================================"
