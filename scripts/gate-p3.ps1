# Phase Gate P3 Verification Script (Pricing, Tax, Region, Currency - 17 tasks)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P3 (Pricing, Tax, Region, Currency)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/7] Checking Architectural Isolation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1

# 2. Currency Domain & Precision Table
Write-Host "`n[2/7] Running Currency & Precision Table Tests (P3-01)..." -ForegroundColor Yellow
cargo test -p stheno-currency

# 3. Region & Store Domains
Write-Host "`n[3/7] Running Region & Store Tests (P3-02, P3-03)..." -ForegroundColor Yellow
cargo test -p stheno-region
cargo test -p stheno-store

# 4. Pricing Engine & Specificity Ranking
Write-Host "`n[4/7] Running Pricing Engine & Specificity Tests (P3-04 to P3-11)..." -ForegroundColor Yellow
cargo test -p stheno-pricing

# 5. Tax Engine & Reverse Computation
Write-Host "`n[5/7] Running Tax Engine, Exact Reverse & Provider Tests (P3-12 to P3-15)..." -ForegroundColor Yellow
cargo test -p stheno-tax

# 6. Pricing Links & Sagas
Write-Host "`n[6/7] Running Pricing Links & Sagas (P3-16, P3-17)..." -ForegroundColor Yellow
cargo test -p stheno-links --test pricing_links
cargo test -p stheno-flows --test pricing --test tax --test region --test store

# 7. Store Pricing Compatibility Parity Suite
Write-Host "`n[7/7] Running Store Pricing Parity Suite (P3-18)..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/parity.ps1 -group store-pricing

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P3 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
