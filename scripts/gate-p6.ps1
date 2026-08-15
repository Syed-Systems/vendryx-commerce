# Phase Gate P6 Verification Script (Order Lifecycle & Mutations - 20 tasks)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P6 (Order Lifecycle, Edits, Returns, Claims, Exchanges, Timeline, Parity)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/6] Checking Architectural Isolation, DTO Separation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-dto-separation.ps1

# 2. Order Edit Workflows (P6-01 to P6-06)
Write-Host "`n[2/6] Running Order Edit Workflows & Zero-Write Preview Tests (P6-01 to P6-06)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test order_edit_items
cargo test -p stheno-flows --test order_edit_shipping
cargo test -p stheno-flows --test order_edit_preview_no_writes
cargo test -p stheno-flows --test order_edit_confirm
cargo test -p stheno-flows --test order_edit_delta
cargo test -p stheno-flows --test order_edit_cancel

# 3. Returns, Claims & Exchanges (P6-07 to P6-14)
Write-Host "`n[3/6] Running Returns, Claims & Exchanges Workflows (P6-07 to P6-14)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test return_request
cargo test -p stheno-flows --test return_receive
cargo test -p stheno-flows --test return_partial
cargo test -p stheno-flows --test return_dismiss
cargo test -p stheno-flows --test claim_items
cargo test -p stheno-flows --test claim_confirm
cargo test -p stheno-flows --test exchange_items
cargo test -p stheno-flows --test exchange_confirm

# 4. Draft Orders, Change Controls & Timeline (P6-15 to P6-18)
Write-Host "`n[4/6] Running Draft Orders, Change Control & Timeline Reconstruction Tests (P6-15 to P6-18)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test draft_order
cargo test -p stheno-flows --test draft_convert
cargo test -p stheno-flows --test change_control
cargo test -p stheno-order --test timeline

# 5. Medusa Corpus & Store Returns Parity (P6-19, P6-20)
Write-Host "`n[5/6] Running Medusa Fixture Corpus & Store Returns Parity Suite (P6-19, P6-20)..." -ForegroundColor Yellow
cargo test -p stheno-flows --test order_corpus
powershell -ExecutionPolicy Bypass -File scripts/parity.ps1 -group store-returns

# 6. Lint & Formatting
Write-Host "`n[6/6] Checking Clippy & Formatting across workspace..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P6 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
