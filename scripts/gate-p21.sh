#!/usr/bin/env bash
# scripts/gate-p21.sh — Phase gate for P21: Provider platform completion (PHASE3_TASKS.md P21)
set -euo pipefail

test -n "${DATABASE_URL:-}" || { echo "ERROR: DATABASE_URL is unset; tests require a real Postgres." >&2; exit 1; }

echo "=========================================================="
echo "          P21 PHASE GATE: PROVIDER PLATFORM COMPLETION   "
echo "=========================================================="

echo "[1/12] Universal gate: Module isolation..."
./scripts/check-module-isolation.sh

echo "[2/12] Universal gate: SQL schema boundaries..."
./scripts/check-sql-boundaries.sh

echo "[3/12] Universal gate: Shared kernel purity..."
./scripts/check-shared-kernel.sh

echo "[4/12] Universal gate: No domain feature gating..."
./scripts/check-no-domain-gates.sh

echo "[5/12] Universal gate: Schema drift..."
./scripts/check-schema-drift.sh

echo "[6/12] Universal gate: PCI SAQ A attestation..."
./scripts/check-pci-saq-a.sh

echo "[7/12] Universal gate: Provider PCI data flow check..."
./scripts/check-provider-pci.sh

echo "[8/12] Universal gate: Provider conformance suite..."
./scripts/check-provider-conformance.sh

echo "[9/12] Inbound webhooks: Payment webhook route & semantics..."
cargo test -p stheno-api-store --test payment_webhook_route
cargo test -p stheno-api-store --test payment_webhook_semantics
cargo test -p stheno-api-store --test raw_body_integrity

echo "[10/12] Inbound webhooks: Fulfillment webhook replay guard..."
cargo test -p stheno-fulfillment --test fulfillment_webhook

echo "[11/12] Provider security: Credentials encryption & tenant isolation under RLS..."
cargo test -p stheno-payment --test credential_encryption
cargo test -p stheno-payment --test provider_tenant_isolation

echo "[12/12] Admin provider management routes..."
cargo test -p stheno-api-admin --test provider_config_routes

echo "=========================================================="
echo "STATUS: GATE P21 PASSED (Exit 0)"
echo "=========================================================="
exit 0
