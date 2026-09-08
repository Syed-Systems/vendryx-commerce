# scripts/load-suite.ps1
# P28-04: Multi-tenant and multi-theme load test runner (PowerShell).

[CmdletBinding()]
param(
    [switch]$MultiTenant
)

$ErrorActionPreference = "Stop"
$RepoRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $RepoRoot

Write-Host "==> Stheno Load Suite (P28-04)..." -ForegroundColor Cyan

Write-Host "1. Running multi-tenant & multi-theme concurrent load test..." -ForegroundColor Yellow
node scripts/multi-tenant-load.js
if ($LASTEXITCODE -ne 0) {
    Write-Error "Multi-tenant load test failed."
    exit 1
}

Write-Host "2. Verifying Core Web Vitals & 0-round-trip SSR branding in headless Chromium..." -ForegroundColor Yellow
npm.cmd --prefix apps/storefront run test:lighthouse
if ($LASTEXITCODE -ne 0) {
    Write-Error "Lighthouse & CWV audit failed."
    exit 1
}

Write-Host "==> Load Suite PASSED: Multi-tenant latency and branding budgets verified." -ForegroundColor Green
exit 0
