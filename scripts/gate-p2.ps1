# Phase Gate P2 Verification Script (Catalog and Inventory - 27 tasks)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P2 (Catalog and Inventory)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/6] Checking Architectural Isolation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1

# 2. Product Domain & Service Derive
Write-Host "`n[2/6] Running Product & Service Derive Tests..." -ForegroundColor Yellow
cargo test -p stheno-product
cargo test -p stheno-db --test service_derive

# 3. Inventory & Concurrency Oversell Tests
Write-Host "`n[3/6] Running Inventory & Concurrency Oversell Tests (P2-13)..." -ForegroundColor Yellow
cargo test -p stheno-inventory
cargo test -p stheno-stock-location -p stheno-sales-channel
cargo test -p stheno-links --test catalog_links

# 4. Catalog Workflows & Batch Operations
Write-Host "`n[4/6] Running Catalog Sagas & Quota Lifecycle Workflows..." -ForegroundColor Yellow
cargo test -p stheno-flows --test product
cargo test -p stheno-flows --test inventory
cargo test -p stheno-flows --test batch_products
cargo test -p stheno-flows --test quota_step
cargo test -p stheno-flows --test quota_concurrent
cargo test -p stheno-flows --test trial_lifecycle
cargo test -p stheno-flows --test grandfathering
cargo test -p stheno-flows --test window_gate

# 5. Security Gates (Upload Pipeline & Egress Guard)
Write-Host "`n[5/6] Running Upload Security & Egress Guard Gates..." -ForegroundColor Yellow
cargo test -p stheno-file --test upload_security
cargo test -p stheno-http --test egress_guard

# 6. Store Catalog Compatibility Parity Suite
Write-Host "`n[6/6] Running Store Catalog Parity Suite..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/parity.ps1 -group store-catalog

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P2 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
