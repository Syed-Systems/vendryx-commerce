#!/usr/bin/env bash
# Phase Gate P6 Verification Script (Order Lifecycle & Mutations - 20 tasks)
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P6 (Order Lifecycle, Edits, Returns, Claims, Exchanges, Timeline, Parity)..."
echo "========================================="

# 1. Architectural Invariants
echo -e "\n[1/6] Checking Architectural Isolation, DTO Separation & Contracts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh

# 2. Order Edit Workflows (P6-01 to P6-06)
echo -e "\n[2/6] Running Order Edit Workflows & Zero-Write Preview Tests (P6-01 to P6-06)..."
cargo test -p stheno-flows --test order_edit_items
cargo test -p stheno-flows --test order_edit_shipping
cargo test -p stheno-flows --test order_edit_preview_no_writes
cargo test -p stheno-flows --test order_edit_confirm
cargo test -p stheno-flows --test order_edit_delta
cargo test -p stheno-flows --test order_edit_cancel

# 3. Returns, Claims & Exchanges (P6-07 to P6-14)
echo -e "\n[3/6] Running Returns, Claims & Exchanges Workflows (P6-07 to P6-14)..."
cargo test -p stheno-flows --test return_request
cargo test -p stheno-flows --test return_receive
cargo test -p stheno-flows --test return_partial
cargo test -p stheno-flows --test return_dismiss
cargo test -p stheno-flows --test claim_items
cargo test -p stheno-flows --test claim_confirm
cargo test -p stheno-flows --test exchange_items
cargo test -p stheno-flows --test exchange_confirm

# 4. Draft Orders, Change Controls & Timeline (P6-15 to P6-18)
echo -e "\n[4/6] Running Draft Orders, Change Control & Timeline Reconstruction Tests (P6-15 to P6-18)..."
cargo test -p stheno-flows --test draft_order
cargo test -p stheno-flows --test draft_convert
cargo test -p stheno-flows --test change_control
cargo test -p stheno-order --test timeline

# 5. Medusa Corpus & Store Returns Parity (P6-19, P6-20)
echo -e "\n[5/6] Running Medusa Fixture Corpus & Store Returns Parity Suite (P6-19, P6-20)..."
cargo test -p stheno-flows --test order_corpus
./scripts/parity.sh --group store-returns

# 6. Lint & Formatting
echo -e "\n[6/6] Checking Clippy & Formatting across workspace..."
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

echo "========================================="
echo "==> Phase Gate P6 PASSED: 100% COMPLETE & VERIFIED!"
echo "========================================="
exit 0
