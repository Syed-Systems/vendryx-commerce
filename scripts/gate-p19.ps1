# Phase Gate P19: Storefront Completion & E2E Verification
$ErrorActionPreference = "Stop"

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P19: Storefront Completion"
Write-Host "========================================================"

Write-Host "`n[1/3] P19-02 to P19-12, P19-14: Storefront E2E test suite..."
npm.cmd --prefix apps/storefront run test:e2e

Write-Host "`n[2/3] P19-13: Storefront Lighthouse and Core Web Vitals audit..."
npm.cmd --prefix apps/storefront run test:lighthouse

Write-Host "`n[3/3] Architectural gate checks..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-isolation.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-sql-boundaries.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-shared-kernel.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-contracts.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-domain-gates.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-in-memory-stores.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"

Write-Host "`n========================================================"
Write-Host "==> Phase Gate P19 PASSED"
Write-Host "========================================================"
