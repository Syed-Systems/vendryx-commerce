#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P4 (Cart, Promotion, Payment)..."
echo "========================================="

echo "[1/8] Checking Architectural Isolation & Contracts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh

echo "[2/8] Verifying Checkout Script Manifest (P4-30, PCI SAQ A)..."
./scripts/check-checkout-scripts.sh

echo "[3/8] Running Cart Engine & Totals Proptests (P4-01 to P4-07)..."
cargo test -p stheno-cart

echo "[4/8] Running Promotion Engine & Budget Concurrency Tests (P4-08 to P4-15)..."
cargo test -p stheno-promotion

echo "[5/8] Running Payment Engine, Ledger Concurrency & Edge Atlas (P4-16 to P4-27)..."
cargo test -p stheno-payment

echo "[6/8] Running Offline & Stripe Payment Providers (P4-23b, P4-24, P4-25)..."
cargo test -p stheno-payment-offline -p stheno-payment-stripe

echo "[7/8] Running Cart, Promotion & Payment Sagas (P4-28)..."
cargo test -p stheno-flows --test cart --test promotion --test payment

echo "[8/8] Running Store Cart Parity Suite (P4-29)..."
./scripts/parity.sh --group store-cart

echo "========================================="
echo "==> Phase Gate P4 PASSED: 100% COMPLETE & VERIFIED!"
echo "========================================="
exit 0
