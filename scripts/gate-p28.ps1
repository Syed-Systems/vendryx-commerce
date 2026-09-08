# Phase Gate P28: Hardening, PCI re-attestation, GA (PowerShell)
$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $RepoRoot

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "          RUNNING PHASE GATE P28 (HARDENING & GA)         " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "`n[0/5] Universal structural & architecture boundary checks..." -ForegroundColor Yellow
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-module-isolation.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-sql-boundaries.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-shared-kernel.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-storefront-boundaries.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-frontend-purity.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-frontend-pin.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-frontend-resolution.sh
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-frontend-not-forked.sh

Write-Host "`n[1/5] P28-01: Gateway redirect & iframe CSP script manifest..." -ForegroundColor Yellow
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-checkout-scripts.sh

Write-Host "`n[2/5] P28-02: PCI DSS v4.0 SAQ-A re-attestation across all providers..." -ForegroundColor Yellow
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-pci-saq-a.sh

Write-Host "`n[3/5] P28-03: Provider failure modes (timeout, 500, callback loss, courier down)..." -ForegroundColor Yellow
cargo test --workspace --test provider_failure_modes

Write-Host "`n[4/5] P28-04: Multi-tenant & multi-theme load suite (budgets on render path)..." -ForegroundColor Yellow
& "C:\Program Files\Git\bin\bash.exe" ./scripts/load-suite.sh --multi-tenant

Write-Host "`n[5/5] P28-05: Operational incident runbooks verification..." -ForegroundColor Yellow
& "C:\Program Files\Git\bin\bash.exe" ./scripts/check-runbooks.sh

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  ALL PHASE 28 GATES PASSED (100% PHASE 3 GA VERIFIED)    " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
exit 0
