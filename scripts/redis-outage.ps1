# Redis Outage Resilience Verification Script (Task P8-08)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Redis Outage Graceful Degradation (Task P8-08)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

Write-Host "[1/2] Running Caching Outage Degradation Test..." -ForegroundColor Yellow
cargo test -p stheno-caching-redis --test redis_caching -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Caching outage degradation check FAILED" -ForegroundColor Red
    exit 1
}

Write-Host "[2/2] Running Distributed Locking Outage Degradation Test..." -ForegroundColor Yellow
cargo test -p stheno-locking --test locking -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Locking outage degradation check FAILED" -ForegroundColor Red
    exit 1
}

Write-Host "=========================================" -ForegroundColor Green
Write-Host "==> Redis Outage Graceful Degradation PASSED: 0 Failed Requests!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
