# Phase Gate P14: Platform Persistence & Link Tables (PowerShell).
$ErrorActionPreference = "Stop"

Write-Host "========================================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P14: Platform Persistence & Link Tables" -ForegroundColor Cyan
Write-Host "========================================================" -ForegroundColor Cyan

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "`n[1/6] Formatting and lints..." -ForegroundColor Yellow
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n[2/6] P14-01 to P14-06: Identity & Access Persistence (customer, auth, user, api-key, rbac)..." -ForegroundColor Yellow
cargo test -p stheno-customer
cargo test -p stheno-auth
cargo test -p stheno-user
cargo test -p stheno-api-key
cargo test -p stheno-rbac
cargo test -p stheno-flows --test authz_matrix

Write-Host "`n[3/6] P14-07 to P14-12: Platform Operations Persistence (file, notification, settings, translation, audit, analytics)..." -ForegroundColor Yellow
cargo test -p stheno-file
cargo test -p stheno-file-local
cargo test -p stheno-notification
cargo test -p stheno-settings
cargo test -p stheno-translation
cargo test -p stheno-audit
cargo test -p stheno-analytics
cargo test -p stheno-flows --test platform_workflows

Write-Host "`n[4/6] P14-13 to P14-17: Search, Index, Caching & Distributed Locking..." -ForegroundColor Yellow
cargo test -p stheno-search
cargo test -p stheno-search-local
cargo test -p stheno-index
cargo test -p stheno-caching
cargo test -p stheno-locking

Write-Host "`n[5/6] P14-18 to P14-21: 22 Link Tables, Query Engine Traversal, Soft-Delete Hiding & GDPR Erasure..." -ForegroundColor Yellow
cargo test -p stheno-links
cargo test -p stheno-query

Write-Host "`n[6/6] P14-22: Zero In-Memory Stores Check..." -ForegroundColor Yellow
& "$PSScriptRoot\check-no-in-memory-stores.ps1"

Write-Host "========================================================" -ForegroundColor Green
Write-Host "==> Phase Gate P14 PASSED" -ForegroundColor Green
Write-Host "========================================================" -ForegroundColor Green
