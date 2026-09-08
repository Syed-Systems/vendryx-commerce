# scripts/gate-p22.ps1 — Phase gate for P22: Bangladesh Payment Gateways (PHASE3_TASKS.md P22)
$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "          P22 PHASE GATE: BANGLADESH PAYMENT GATEWAYS     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host "[1/8] Universal gates: Module isolation, boundaries, and kernel purity..."
& "C:\Program Files\Git\bin\sh.exe" scripts/check-module-isolation.sh
& "C:\Program Files\Git\bin\sh.exe" scripts/check-sql-boundaries.sh
& "C:\Program Files\Git\bin\sh.exe" scripts/check-shared-kernel.sh
& "C:\Program Files\Git\bin\sh.exe" scripts/check-no-domain-gates.sh
& "C:\Program Files\Git\bin\sh.exe" scripts/check-schema-drift.sh
& "C:\Program Files\Git\bin\sh.exe" scripts/check-provider-pci.sh
python scripts/check-no-fabricated-handlers.py

Write-Host "[2/8] P22-01..P22-06: bKash Tokenized Checkout suite..."
cargo test -p stheno-payment-bkash --test registration
cargo test -p stheno-payment-bkash --test token_lifecycle
cargo test -p stheno-payment-bkash --test initiate
cargo test -p stheno-payment-bkash --test capture
cargo test -p stheno-payment-bkash --test webhook
cargo test -p stheno-payment-bkash --test refund

Write-Host "[3/8] P22-07..P22-11: Nagad DFS Checkout suite..."
cargo test -p stheno-payment-nagad --test registration
cargo test -p stheno-payment-nagad --test initialise
cargo test -p stheno-payment-nagad --test capture
cargo test -p stheno-payment-nagad --test webhook
cargo test -p stheno-payment-nagad --test refund

Write-Host "[4/8] P22-12..P22-15: SSLCommerz Hosted Session & IPN suite..."
cargo test -p stheno-payment-sslcommerz --test registration
cargo test -p stheno-payment-sslcommerz --test session
cargo test -p stheno-payment-sslcommerz --test ipn
cargo test -p stheno-payment-sslcommerz --test refund

Write-Host "[5/8] P22-16: All three providers pass provider conformance unmodified..."
& "C:\Program Files\Git\bin\sh.exe" scripts/check-provider-conformance.sh

Write-Host "[6/8] P22-17: BDT end to end: decimal totals & minor-unit conversion invariants..."
cargo test -p stheno-payment --test bdt_money

Write-Host "[7/8] P22-18: Recorded wiremock sandbox fixtures verification..."
& "C:\Program Files\Git\bin\sh.exe" scripts/check-provider-fixtures.sh

Write-Host "[8/8] Workspace build and compiler hygiene..."
cargo check --workspace

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "STATUS: GATE P22 PASSED (Exit 0)" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
exit 0
