Write-Host "==> Checking for domain feature gating..." -ForegroundColor Cyan

$matches = Get-ChildItem -Path "crates/modules" -Recurse -Filter "*.rs" | Select-String -Pattern "Entitlements" -ErrorAction SilentlyContinue

if ($matches) {
    Write-Host "ERROR: Entitlements referenced inside crates/modules/ (AGENTS.md §2.4):" -ForegroundColor Red
    $matches | ForEach-Object { Write-Host "$($_.Path):$($_.LineNumber): $($_.Line)" -ForegroundColor Red }
    exit 1
}

Write-Host "Domain feature gating check PASSED: No Entitlements references in domain modules." -ForegroundColor Green
exit 0
