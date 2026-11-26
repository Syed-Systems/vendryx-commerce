#!/usr/bin/env bash
# Phase Gate P13: Module persistence on Postgres.
#
# Runs every P13 exit command in dependency order.
# Enforces module migration, transactional integrity, idempotency, and parity against Postgres.
set -euo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P13: Module persistence on Postgres"
echo "========================================================"

if [ -z "${DATABASE_URL:-}" ]; then
    echo "FAIL: DATABASE_URL is unset. Every assertion below needs a real database." >&2
    echo "      export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno" >&2
    exit 1
fi

echo -e "\n[1/8] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/8] P13-01 to P13-05: Workflow Postgres Store & Crash Recovery..."
cargo test -p stheno-workflow --test store_schema
cargo test -p stheno-workflow --test durable_store
cargo test -p stheno-workflow --test lease_contention
cargo test -p stheno-workflow --test compensation_durability
./scripts/kill-resume.sh

echo -e "\n[3/8] P13-06 to P13-10: Cart & Promotion Persistence & Concurrency..."
cargo test -p stheno-cart --test schema
cargo test -p stheno-cart
cargo test -p stheno-promotion --test schema
cargo test -p stheno-promotion
cargo test -p stheno-promotion --test budget_concurrency

echo -e "\n[4/8] P13-11 to P13-14: Payment Persistence & Idempotency..."
cargo test -p stheno-payment --test schema
cargo test -p stheno-payment
cargo test -p stheno-payment --test idempotency_durability
cargo test -p stheno-payment --test outbound_idempotency
cargo test -p stheno-api-store --test payment_parity

echo -e "\n[5/8] P13-15 to P13-18: Order Schema, Versioning & Order Change..."
cargo test -p stheno-order --test schema
cargo test -p stheno-order
cargo test -p stheno-order --test versioning
cargo test -p stheno-order-change

echo -e "\n[6/8] P13-19 to P13-20: Fulfillment Schema & Service..."
cargo test -p stheno-fulfillment --test schema
cargo test -p stheno-fulfillment

echo -e "\n[7/8] P13-21 to P13-23: Checkout Atomicity, Completion Race & Money Precision..."
cargo test -p stheno-flows --test checkout_atomicity
cargo test -p stheno-flows --test cart_completion_race
cargo test -p stheno-order --test money_precision_db

echo -e "\n[8/8] P13-24 to P13-25: Store & Admin Parity against Postgres..."
./scripts/parity.sh --group store-cart
./scripts/parity.sh --group store-order
./scripts/parity.sh --group admin-orders
./scripts/parity.sh --group admin-payments

echo "========================================================"
echo "==> Phase Gate P13 PASSED"
echo "========================================================"
