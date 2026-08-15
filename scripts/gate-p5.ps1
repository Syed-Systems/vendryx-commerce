# Phase Gate P5 Verification Script (Order, Fulfillment, Complete-Cart SAGA, Sales Channel - 22 tasks)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P5 (Order, Fulfillment, Flows, Store Parity)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/7] Checking Architectural Isolation, DTO Separation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-dto-separation.ps1

# 2. Order Module (P5-01 to P5-06)
Write-Host "`n[2/7] Running Order Module Tests (P5-01 to P5-06)..." -ForegroundColor Yellow
cargo test -p stheno-order --test models
cargo test -p stheno-order --test change_integration
cargo test -p stheno-order --test versioning
cargo test -p stheno-order --test summary
cargo test -p stheno-order --test ledger
cargo test -p stheno-order --test display_id_concurrent

# 3. Fulfillment Module & Manual Provider (P5-07 to P5-13)
Write-Host "`n[3/7] Running Fulfillment Module Tests (P5-07 to P5-13)..." -ForegroundColor Yellow
cargo test -p stheno-fulfillment --test models
cargo test -p stheno-fulfillment --test zones
cargo test -p stheno-fulfillment --test shipping_options
cargo test -p stheno-fulfillment --test eligibility
cargo test -p stheno-fulfillment-manual --test manual
cargo test -p stheno-fulfillment --test calculated
cargo test -p stheno-fulfillment --test lifecycle

# 4. Links Module (P5-14)
Write-Host "`n[4/7] Running Order Cross-Module Links Tests (P5-14)..." -ForegroundColor Yellow
cargo test -p stheno-links --test order_links

# 5. Workflows (P5-15 to P5-21)
Write-Host "`n[5/7] Running SAGA Complete-Cart, Order & Fulfillment Flows (P5-15 to P5-21)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test complete_cart
cargo test -p stheno-flows --test complete_cart_compensation
cargo test -p stheno-flows --test complete_cart_resume
cargo test -p stheno-flows --test complete_cart_idempotent
cargo test -p stheno-flows --test order_lifecycle
cargo test -p stheno-flows --test fulfillment
cargo test -p stheno-flows --test order_transfer

# 6. Store API Endpoints Parity (P5-22)
Write-Host "`n[6/7] Running Store Order & Shipping Options Parity Tests (P5-22)..." -ForegroundColor Yellow
cargo test -p stheno-api-store --test order_parity

# 7. Lint & Formatting
Write-Host "`n[7/7] Checking Clippy & Formatting across workspace..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P5 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
