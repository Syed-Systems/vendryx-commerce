# Phase Gate P12: Persistence for catalog, pricing, inventory, config (PowerShell).
$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P12: Catalog, Pricing, Inventory, Config Persistence" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "`n[1/7] Formatting and lints..." -ForegroundColor Yellow
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n[2/7] P12-01 to P12-05: Pricing persistence & hot path..." -ForegroundColor Yellow
cargo test -p stheno-pricing --test schema
cargo test -p stheno-pricing

Write-Host "`n[3/7] P12-06 to P12-08: Inventory persistence, no-oversell & concurrent reservations..." -ForegroundColor Yellow
cargo test -p stheno-inventory --test schema
cargo test -p stheno-inventory --test inventory
cargo test -p stheno-inventory --test reservations
cargo test -p stheno-inventory --test no_oversell
cargo test -p stheno-inventory --test concurrent_reservation

Write-Host "`n[4/7] P12-09 to P12-12: Stock Location, Sales Channel, Tax, Store..." -ForegroundColor Yellow
cargo test -p stheno-stock-location
cargo test -p stheno-sales-channel
cargo test -p stheno-tax
cargo test -p stheno-store

Write-Host "`n[5/7] P12-13 to P12-14: Product variant database uniqueness..." -ForegroundColor Yellow
cargo test -p stheno-product --test variant_identity
cargo test -p stheno-product --test variant_identity_concurrent

Write-Host "`n[6/7] P12-19 to P12-20: Store Parity against Postgres..." -ForegroundColor Yellow
& "$PSScriptRoot\parity.ps1" -group store-catalog
& "$PSScriptRoot\parity.ps1" -group store-pricing

Write-Host "`n[7/7] P12-21 to P12-24: Admin Parity against Postgres..." -ForegroundColor Yellow
& "$PSScriptRoot\parity.ps1" -group admin-catalog
& "$PSScriptRoot\parity.ps1" -group admin-inventory
& "$PSScriptRoot\parity.ps1" -group admin-pricing
& "$PSScriptRoot\parity.ps1" -group admin-config

Write-Host "========================================================" -ForegroundColor Green
Write-Host "==> Phase Gate P12 PASSED" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
