# P13-05: Kill-resume against real restarts and durable Postgres stores.
$ErrorActionPreference = "Stop"

$runs = if ($env:RUNS) { [int]$env:RUNS } else { 10 }
Write-Host "==> Running Kill-and-Resume harness ($runs runs) (P13-05)..."

for ($i = 1; $i -le $runs; $i++) {
    & cargo test -p stheno-workflow --test durable_store --test compensation_durability --test lease_contention --test idempotency_concurrent --test compensation_matrix --quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Kill-and-Resume run $i failed with exit code $LASTEXITCODE"
        exit $LASTEXITCODE
    }
}

Write-Host "Kill-and-Resume harness PASSED: $runs/$runs crash recovery and restart runs completed successfully."
exit 0
