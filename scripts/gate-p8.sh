#!/usr/bin/env bash
# Phase Gate P8: Platform Services Verification Script
# Strict build & CI enforcement per AGENTS.md §7

set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P8: Platform Services..."
echo "========================================="

# 1. Formatting & Linting Check
echo -e "\n[1/8] Checking Cargo Formatting & Clippy across workspace..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# 2. File Module & Storage Providers (P8-01, P8-02)
echo -e "\n[2/8] Verifying File Module, Magic-Byte Sniffing, SVG Rejection, Local & S3 Providers..."
cargo test -p stheno-file -p stheno-file-local -p stheno-file-s3

# 3. Notification Module, Multi-Channel Routing, VAPID WebPush & Social Push Suppression (P8-03, P8-04, P8-04b, P8-22d)
echo -e "\n[3/8] Verifying Notification Module, SendGrid, WebPush & Social Push Suppression..."
cargo test -p stheno-notification -p stheno-notification-sendgrid -p stheno-notification-webpush
cargo test -p stheno-subscribers --test push_order_placed

# 4. Caching, Distributed Locking & Outage Graceful Degradation (P8-05, P8-06, P8-07, P8-08)
echo -e "\n[4/8] Verifying Caching, Redis, Distributed Locking & Outage Resilience..."
cargo test -p stheno-caching -p stheno-caching-redis -p stheno-locking
bash scripts/redis-outage.sh

# 5. Search Engine, Full-Text Provider, Event Sync & Convergence (P8-09, P8-10, P8-11)
echo -e "\n[5/8] Verifying Search Engine, Event-Driven Sync & 1,000-Product Convergence Benchmark..."
cargo test -p stheno-search -p stheno-search-local

# 6. Denormalized Indexing, Rebuild, Lag Metrics & Cross-Module Filters (P8-12, P8-13, P8-14)
echo -e "\n[6/8] Verifying Denormalized Indexing, Rebuilds, Lag Tracking & Cross-Module Filters..."
cargo test -p stheno-index

# 7. Translations, Settings & Analytics with PII Redaction (P8-15, P8-16, P8-17)
echo -e "\n[7/8] Verifying Translations, Settings & Analytics with PII Redaction..."
cargo test -p stheno-translation -p stheno-settings -p stheno-analytics

# 8. Privacy & Erasure Cascade, Data Export, Reconciliation & 13 Platform Workflows (P8-18 to P8-23)
echo -e "\n[8/8] Verifying Privacy Cascade, Retention, Data Export, Reconciliation & Platform Workflows..."
cargo test -p stheno-payment --test reconciliation
cargo test -p stheno-flows --test erasure_cascade --test erasure_verification --test retention_jobs --test data_export --test reporting --test reporting_window --test platform_workflows

echo "========================================="
echo "==> Phase Gate P8 PASSED: All 27 Platform Service Tasks Verified Cleanly!"
echo "========================================="
