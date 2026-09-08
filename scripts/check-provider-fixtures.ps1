# scripts/check-provider-fixtures.ps1 — Recorded sandbox fixtures for BD payment providers (P22-18)
$ErrorActionPreference = "Stop"

Write-Host "==> Verifying recorded sandbox fixtures for BD payment providers..."

Write-Host "[1/3] Verifying bKash wiremock sandbox fixtures..."
cargo test -p stheno-payment-bkash --test bkash_flow
cargo test -p stheno-payment-bkash --test initiate
cargo test -p stheno-payment-bkash --test capture
cargo test -p stheno-payment-bkash --test refund

Write-Host "[2/3] Verifying SSLCommerz wiremock sandbox fixtures..."
cargo test -p stheno-payment-sslcommerz --test sslcommerz_flow
cargo test -p stheno-payment-sslcommerz --test session
cargo test -p stheno-payment-sslcommerz --test ipn
cargo test -p stheno-payment-sslcommerz --test refund

Write-Host "[3/3] Verifying Nagad wiremock sandbox fixtures..."
cargo test -p stheno-payment-nagad --test initialise
cargo test -p stheno-payment-nagad --test capture
cargo test -p stheno-payment-nagad --test refund
cargo test -p stheno-payment-nagad --test webhook

Write-Host "All recorded sandbox fixtures passed without external calls (Exit 0)." -ForegroundColor Green
exit 0
