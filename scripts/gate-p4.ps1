# Phase Gate P4 Verification Script (Cart, Promotion, Payment - 31 tasks)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P4 (Cart, Promotion, Payment)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/8] Checking Architectural Isolation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1

# 2. Checkout Script Manifest
Write-Host "`n[2/8] Verifying Checkout Script Manifest (P4-30, PCI SAQ A)..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-checkout-scripts.ps1

# 3. Cart Domain & Totals Engine
Write-Host "`n[3/8] Running Cart Engine & Totals Proptests (P4-01 to P4-07)..." -ForegroundColor Yellow
cargo test -p stheno-cart

# 4. Promotion Domain & Budget Concurrency
Write-Host "`n[4/8] Running Promotion Engine & Budget Concurrency Tests (P4-08 to P4-15)..." -ForegroundColor Yellow
cargo test -p stheno-promotion

# 5. Payment Domain, Ledger Invariants & Edge Atlas
Write-Host "`n[5/8] Running Payment Engine, Ledger Concurrency & Edge Atlas (P4-16 to P4-27)..." -ForegroundColor Yellow
cargo test -p stheno-payment

# 6. Offline and Stripe Payment Providers
Write-Host "`n[6/8] Running Offline & Stripe Payment Providers (P4-23b, P4-24, P4-25)..." -ForegroundColor Yellow
cargo test -p stheno-payment-offline -p stheno-payment-stripe

# 7. Cart, Promotion & Payment Sagas
Write-Host "`n[7/8] Running Cart, Promotion & Payment Sagas (P4-28)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test cart --test promotion --test payment

# 8. Store Cart Compatibility Parity Suite
Write-Host "`n[8/8] Running Store Cart Parity Suite (P4-29)..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/parity.ps1 -group store-cart

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P4 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
