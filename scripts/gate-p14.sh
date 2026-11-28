#!/usr/bin/env bash
# Phase Gate P14: Platform Persistence & Link Tables (Bash).
set -euo pipefail

echo "========================================================"
echo "==> Verifying Phase Gate P14: Platform Persistence & Link Tables"
echo "========================================================"

export DATABASE_URL="${DATABASE_URL:-postgres://stheno:stheno@localhost:5432/stheno}"

echo -e "\n[1/6] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/6] P14-01 to P14-06: Identity & Access Persistence (customer, auth, user, api-key, rbac)..."
cargo test -p stheno-customer
cargo test -p stheno-auth
cargo test -p stheno-user
cargo test -p stheno-api-key
cargo test -p stheno-rbac
cargo test -p stheno-flows --test authz_matrix

echo -e "\n[3/6] P14-07 to P14-12: Platform Operations Persistence (file, notification, settings, translation, audit, analytics)..."
cargo test -p stheno-file
cargo test -p stheno-file-local
cargo test -p stheno-notification
cargo test -p stheno-settings
cargo test -p stheno-translation
cargo test -p stheno-audit
cargo test -p stheno-analytics
cargo test -p stheno-flows --test platform_workflows

echo -e "\n[4/6] P14-13 to P14-17: Search, Index, Caching & Distributed Locking..."
cargo test -p stheno-search
cargo test -p stheno-search-local
cargo test -p stheno-index
cargo test -p stheno-caching
cargo test -p stheno-locking

echo -e "\n[5/6] P14-18 to P14-21: 22 Link Tables, Query Engine Traversal, Soft-Delete Hiding & GDPR Erasure..."
cargo test -p stheno-links
cargo test -p stheno-query

echo -e "\n[6/6] P14-22: Zero In-Memory Stores Check..."
./scripts/check-no-in-memory-stores.sh

echo "========================================================"
echo "==> Phase Gate P14 PASSED"
echo "========================================================"
