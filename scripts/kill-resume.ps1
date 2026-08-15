param(
    [int]$runs = 100
)

Write-Host "==> Running Kill-and-Resume harness ($runs runs) (P1-14)..." -ForegroundColor Cyan

$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

for ($i = 1; $i -le $runs; $i++) {
    & cargo test -p stheno-workflow --test idempotency_concurrent --test compensation_matrix --quiet *>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Kill-and-Resume failed at run $i" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Kill-and-Resume harness PASSED: $runs/$runs crash recovery runs completed successfully." -ForegroundColor Green
exit 0
