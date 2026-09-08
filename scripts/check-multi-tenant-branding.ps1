# scripts/check-multi-tenant-branding.ps1
# P26-08: Two tenants, one process, different renders.

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "==> Verifying multi-tenant branding isolation & distinct renders (P26-08)..." -ForegroundColor Cyan

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}

Write-Host "1. Checking backend RLS multi-tenant branding isolation..." -ForegroundColor Yellow
cargo test -p stheno-api-store --test branding_endpoint multi_tenant_branding_isolation
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "2. Checking storefront runtime token resolution & CSS properties..." -ForegroundColor Yellow
npm.cmd --prefix apps/storefront run test -- branding-tokens
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Multi-tenant branding check PASSED: Distinct renders per tenant verified." -ForegroundColor Green
exit 0
