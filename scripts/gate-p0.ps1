Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Running Phase P0 Gate Suite..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "1. Checking formatting..." -ForegroundColor Yellow
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "2. Running Clippy..." -ForegroundColor Yellow
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "3. Running all workspace tests..." -ForegroundColor Yellow
cargo test --workspace
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "4. Running Boundary Scripts..." -ForegroundColor Yellow
& ./scripts/check-module-isolation.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-sql-boundaries.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-dto-separation.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-shared-kernel.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-no-domain-gates.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-module-contracts.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-migration-phases.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }
& ./scripts/check-schema-drift.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "5. Running Negative Boundary Tests (P0-25)..." -ForegroundColor Yellow
& ./scripts/test-boundary-checks.ps1
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "=========================================" -ForegroundColor Green
Write-Host "==> Phase P0 Gate PASSED: 100% GREEN!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
