Write-Host "==> Checking shared kernel (stheno-types)..." -ForegroundColor Cyan

$pattern = "struct Product|struct Order|struct Cart|struct Customer|struct Payment"
$matches = Select-String -Path "crates/core/stheno-types/src/*.rs" -Pattern $pattern -ErrorAction SilentlyContinue

if ($matches) {
    Write-Host "ERROR: Domain entities found in stheno-types (AGENTS.md §2.3):" -ForegroundColor Red
    $matches | ForEach-Object { Write-Host $_.Line -ForegroundColor Red }
    exit 1
}

Write-Host "Shared kernel check PASSED: stheno-types contains primitives only." -ForegroundColor Green
exit 0
