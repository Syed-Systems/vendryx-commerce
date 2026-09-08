# scripts/gate-p24.ps1 — Phase gate for P24: COD settlement and reconciliation (PHASE3_TASKS.md P24)
$ErrorActionPreference = "Stop"

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "          P24 PHASE GATE: COD SETTLEMENT & RECONCILIATION " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

Write-Host "[1/10] Universal gates: Module isolation, boundaries, and kernel purity..."
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

& "C:\Program Files\Git\bin\sh.exe" scripts/check-schema-drift.sh
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

python scripts/check-no-fabricated-handlers.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[2/10] P24-01: Courier settlement tables RLS audit..."
cargo test -p stheno-db --test rls_audit
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[3/10] P24-02: Order three distinct amounts (order total, collected, remitted)..."
cargo test -p stheno-order --test cod_amounts
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[4/10] P24-03..P24-04: Delivery confirmation capture (BP6) and returns..."
cargo test -p stheno-payment --test cod_capture_on_delivery
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p stheno-payment --test cod_return_no_capture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[5/10] P24-05: Settlement report import idempotency (BP7)..."
cargo test -p stheno-payment --test settlement_import
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[6/10] P24-06..P24-07: Reconciliation buckets & courier fee balance identity (BP8)..."
cargo test -p stheno-payment --test reconciliation_buckets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p stheno-payment --test settlement_balance
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[7/10] P24-08..P24-09: Settlement mismatch & aged line discrepancy alerting..."
cargo test -p stheno-payment --test settlement_mismatch
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p stheno-payment --test settlement_alerting
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[8/10] P24-10: Admin reconciliation routes..."
cargo test -p stheno-api-admin --test reconciliation_routes
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[9/10] P24-11: Admin reconciliation UI..."
cmd.exe /c "npm --prefix apps/admin run test -- reconciliation"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[10/10] P24-12: End-to-end COD lifecycle script..."
powershell -ExecutionPolicy Bypass -File scripts/cod-lifecycle.ps1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "STATUS: GATE P24 PASSED (Exit 0)" -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
exit 0
