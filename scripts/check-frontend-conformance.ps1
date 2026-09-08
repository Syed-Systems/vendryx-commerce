# scripts/check-frontend-conformance.ps1 — P27-02 Theme Conformance Gate
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

Write-Host "==> Running theme conformance gate against frontend-default..."
& npm.cmd --prefix apps/storefront run test:conformance -- --frontend=default
if ($LASTEXITCODE -ne 0) {
    Write-Error "Theme conformance check failed"
    exit 1
}

Write-Host "==> Conformance gate passed for frontend-default."
exit 0
