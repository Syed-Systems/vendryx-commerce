Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Running Phase P1 Gate Suite..." -ForegroundColor Cyan
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

Write-Host "4. Running P1 Specific Gates..." -ForegroundColor Yellow
# P1-08: Compensation matrix
cargo test -p stheno-workflow --test compensation_matrix
if ($LASTEXITCODE -ne 0) { exit 1 }
# P1-14: Kill and resume harness
& ./scripts/kill-resume.ps1 -runs 20
if ($LASTEXITCODE -ne 0) { exit 1 }
# P1-15: Idempotency concurrent
cargo test -p stheno-workflow --test idempotency_concurrent
if ($LASTEXITCODE -ne 0) { exit 1 }
# P1-20: Zero N+1 queries
cargo test -p stheno-query --test no_n_plus_one
if ($LASTEXITCODE -ne 0) { exit 1 }
# P1-23: Order change corpus
cargo test -p stheno-order-change --test corpus
if ($LASTEXITCODE -ne 0) { exit 1 }
# P1-24: Composition root
cargo run -p stheno-server -- --check-config
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host "=========================================" -ForegroundColor Green
Write-Host "==> Phase P1 Gate PASSED: 100% GREEN!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
