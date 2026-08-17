# Phase Gate P13: Module persistence on Postgres (PowerShell).
$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P13: Module persistence on Postgres" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

if (-not $env:DATABASE_URL) {
    Write-Error "DATABASE_URL is unset. Set DATABASE_URL before running gate-p13."
    exit 1
}

Write-Host "`n[1/8] Formatting and lints..." -ForegroundColor Yellow
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n[2/8] P13-01 to P13-05: Workflow Postgres Store & Crash Recovery..." -ForegroundColor Yellow
cargo test -p stheno-workflow --test store_schema
cargo test -p stheno-workflow --test durable_store
cargo test -p stheno-workflow --test lease_contention
cargo test -p stheno-workflow --test compensation_durability
& "$PSScriptRoot\kill-resume.ps1"

Write-Host "`n[3/8] P13-06 to P13-10: Cart & Promotion Persistence & Concurrency..." -ForegroundColor Yellow
cargo test -p stheno-cart --test schema
cargo test -p stheno-cart
cargo test -p stheno-promotion --test schema
cargo test -p stheno-promotion
cargo test -p stheno-promotion --test budget_concurrency

Write-Host "`n[4/8] P13-11 to P13-14: Payment Persistence & Idempotency..." -ForegroundColor Yellow
cargo test -p stheno-payment --test schema
cargo test -p stheno-payment
cargo test -p stheno-payment --test idempotency_durability
cargo test -p stheno-payment --test outbound_idempotency
cargo test -p stheno-api-store --test payment_parity

Write-Host "`n[5/8] P13-15 to P13-18: Order Schema, Versioning & Order Change..." -ForegroundColor Yellow
cargo test -p stheno-order --test schema
cargo test -p stheno-order
cargo test -p stheno-order --test versioning
cargo test -p stheno-order-change

Write-Host "`n[6/8] P13-19 to P13-20: Fulfillment Schema & Service..." -ForegroundColor Yellow
cargo test -p stheno-fulfillment --test schema
cargo test -p stheno-fulfillment

Write-Host "`n[7/8] P13-21 to P13-23: Checkout Atomicity, Completion Race & Money Precision..." -ForegroundColor Yellow
cargo test -p stheno-flows --test checkout_atomicity
cargo test -p stheno-flows --test cart_completion_race
cargo test -p stheno-order --test money_precision_db

Write-Host "`n[8/8] P13-24 to P13-25: Store & Admin Parity against Postgres..." -ForegroundColor Yellow
& "$PSScriptRoot\parity.ps1" -Group store-cart
& "$PSScriptRoot\parity.ps1" -Group store-order
& "$PSScriptRoot\parity.ps1" -Group admin-orders
& "$PSScriptRoot\parity.ps1" -Group admin-payments

Write-Host "========================================================" -ForegroundColor Green
Write-Host "==> Phase Gate P13 PASSED" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
