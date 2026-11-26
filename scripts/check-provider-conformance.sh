#!/usr/bin/env bash
# scripts/check-provider-conformance.sh — Provider conformance and capability honesty suite (P21-08..P21-10)
set -euo pipefail

echo "==> Running payment provider conformance suite..."

cargo test -p stheno-payment --test conformance_suite -- --nocapture
cargo test -p stheno-payment --test conformance_capabilities -- --nocapture
cargo test -p stheno-payment --test capabilities_redirect -- --nocapture
cargo test -p stheno-payment --test redirect_contract -- --nocapture

echo "Provider conformance check PASSED: all providers satisfy lifecycle, idempotency, and capability honesty."
exit 0
