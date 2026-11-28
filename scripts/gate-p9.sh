#!/usr/bin/env bash
# Phase Gate P9: Admin API, entitlements, and dashboard verification
# Strict build & CI enforcement per AGENTS.md §7
#
# This gate reports the real state of P9. Where a task's deliverable does not exist,
# the gate fails and says so rather than skipping quietly.

set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P9: Admin API & Dashboard..."
echo "========================================="

echo -e "\n[1/6] Checking Cargo formatting & Clippy across workspace..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/6] Verifying Admin response envelopes in Medusa's exact shapes (P9-01)..."
cargo test -p stheno-api-core --test envelopes

echo -e "\n[3/6] Verifying Admin route parity across all resource groups (P9-02 to P9-13)..."
bash scripts/parity.sh --group admin-catalog
bash scripts/parity.sh --group admin-inventory
bash scripts/parity.sh --group admin-pricing
bash scripts/parity.sh --group admin-orders
bash scripts/parity.sh --group admin-customers
bash scripts/parity.sh --group admin-marketing
bash scripts/parity.sh --group admin-config

echo -e "\n[4/6] Verifying feature flags, entitlements & conditional mounting (P9-14b to P9-14e)..."
cargo test -p stheno-api-admin-platform --test entitlements
cargo test -p stheno-api-admin-platform --test workflow_obs
cargo test -p stheno-api-router --test gated_mounting
cargo test -p stheno-api-router --test tier_matrix

echo -e "\n[5/6] Verifying no domain-level feature gating (ENTITLEMENTS.md §6.1)..."
bash scripts/check-no-domain-gates.sh
bash scripts/check-no-enterprise-paths.sh

echo -e "\n[6/6] Verifying social draft orders and the three draft-order traps (P9-32, P9-33)..."
cargo test -p stheno-flows-order --test social_draft
cargo test -p stheno-flows-order --test social_draft_convert

echo -e "\n--- Dashboard deliverables (P9-15 to P9-31, P9-34, P9-35) ---"
# ADR-002 requires a fork of Medusa's packages/admin/dashboard. Until that exists there
# is nothing to build or e2e, and this gate must not pretend otherwise.
if [ ! -f apps/admin/package.json ] || ! grep -q '"scripts"' apps/admin/package.json; then
    echo "FAIL: apps/admin has no runnable app." >&2
    echo "      ADR-002 specifies forking packages/admin/dashboard. What exists today is" >&2
    echo "      component sources only -- no entry point, router, bundler config, or deps." >&2
    echo "      P9 dashboard tasks (P9-15..P9-31, P9-34, P9-35) are UNVERIFIED." >&2
    exit 1
fi
cd apps/admin && npm ci && npm run build && npm run test:e2e

echo "========================================="
echo "==> Phase Gate P9 PASSED"
echo "========================================="
