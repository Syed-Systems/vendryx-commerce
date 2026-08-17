Write-Host "==> Checking DTO / Entity separation..." -ForegroundColor Cyan

# 1. Ensure models are strictly internal (never pub use re-exported in dto.rs or lib.rs)
$reexports = Get-ChildItem -Path "crates/modules" -Recurse -Filter "*.rs" | Select-String -Pattern "pub use.*models"
if ($reexports) {
    Write-Host "ERROR: Entity models must never be re-exported across the module boundary (AGENTS.md §3.4):" -ForegroundColor Red
    $reexports | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    exit 1
}

# 2. Ensure API handlers in crates/api do not take internal models directly
$apiLeaks = Get-ChildItem -Path "crates/api" -Recurse -Filter "*.rs" | Select-String -Pattern "Json<.*models::"
if ($apiLeaks) {
    Write-Host "ERROR: API handlers must use Request DTOs, never entity models (AGENTS.md §2.9):" -ForegroundColor Red
    $apiLeaks | ForEach-Object { Write-Host $_ -ForegroundColor Red }
    exit 1
}

Write-Host "DTO separation check PASSED: Entity models strictly isolated from DTO boundary." -ForegroundColor Green
exit 0
