# Phase Gate P27: Theme packages & conformance
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

Write-Host "=========================================================="
Write-Host "          RUNNING PHASE GATE P27 (THEME CONFORMANCE)       "
Write-Host "=========================================================="

Write-Host ""
Write-Host "[0/10] Storefront structural boundaries..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-storefront-boundaries.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-purity.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-pin.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-resolution.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-not-forked.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-switch.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[1/10] P27-01: Theme conformance e2e pack (frontend-default)..."
& npm.cmd --prefix apps/storefront run test:conformance -- --frontend=default
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[2/10] P27-02: Conformance gate script..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-conformance.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[3/10] P27-03: Storefront typecheck & contract conformance..."
& npm.cmd --prefix apps/storefront run typecheck
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[4/10] P27-04: Theme B catalog and product surfaces..."
& npm.cmd --prefix apps/storefront run test:conformance -- --frontend=b
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[5/10] P27-05: Theme B cart and account surfaces..."
& npm.cmd --prefix apps/storefront run test:conformance -- --frontend=b
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[6/10] P27-06: Checkout skinning, script manifest & ownership..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-checkout-scripts.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }
& npm.cmd --prefix apps/storefront run test -- checkout-ownership
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[7/10] P27-07: Core delta budget audit..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-core-delta-budget.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[8/10] P27-08: Per-tenant themed build & deploy pipeline..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/deploy-themed-storefront.sh --dry-run"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[9/10] P27-09: Bundler frontend switch verification..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-switch.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "[10/10] P27-10: Theme authoring documentation..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-frontend-docs.sh"
if ($LASTEXITCODE -ne 0) { exit 1 }

Write-Host ""
Write-Host "=========================================================="
Write-Host "STATUS: GATE P27 PASSED (Exit 0)"
Write-Host "=========================================================="
exit 0
