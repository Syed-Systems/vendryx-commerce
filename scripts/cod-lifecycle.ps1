# scripts/cod-lifecycle.ps1 — End-to-end COD lifecycle (PHASE3_TASKS.md P24-12)
$ErrorActionPreference = "Stop"

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "=========================================================="
Write-Host "          P24-12: END-TO-END COD LIFECYCLE                "
Write-Host "=========================================================="

Write-Host "[1/4] Running End-to-End COD Lifecycle Suite..."
cargo test -p stheno-payment --test cod_lifecycle
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[2/4] Verifying Capture on Delivery (BP6)..."
cargo test -p stheno-payment --test cod_capture_on_delivery
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[3/4] Verifying Courier Fee Balance Identity sum(remitted) + sum(fees) == sum(collected) (BP8)..."
cargo test -p stheno-payment --test settlement_balance
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "[4/4] Verifying Return Handling with Net-Zero Expectation..."
cargo test -p stheno-payment --test cod_return_no_capture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "=========================================================="
Write-Host "STATUS: END-TO-END COD LIFECYCLE PASSED (Exit 0)"
Write-Host "=========================================================="
exit 0
