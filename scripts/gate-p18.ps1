# Phase Gate P18: Admin Dashboard Fork & Verification
$ErrorActionPreference = "Stop"

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P18: Admin Dashboard Fork"
Write-Host "========================================================"

Write-Host "`n[1/4] P18-02: Checking for zero enterprise paths..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-enterprise-paths.sh"

Write-Host "`n[2/4] P18-03 to P18-27: Admin dashboard E2E test suite..."
npm.cmd --prefix apps/admin run test:e2e

Write-Host "`n[3/4] P18-28: Admin dashboard WCAG 2.2 AA accessibility audit..."
npm.cmd --prefix apps/admin run test:a11y

Write-Host "`n[4/4] Architectural gate checks..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-isolation.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-sql-boundaries.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-shared-kernel.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-contracts.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-domain-gates.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-in-memory-stores.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"

Write-Host "`n========================================================"
Write-Host "==> Phase Gate P18 PASSED"
Write-Host "========================================================"
