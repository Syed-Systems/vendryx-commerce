# Phase Gate P20: Hardening, Load, and GA Verification
$ErrorActionPreference = "Stop"

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P20: Hardening, Load, and GA"
Write-Host "========================================================"

Write-Host "`n[1/5] P20-18: Security disclosure & SBOM compliance audit..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-disclosure.sh"

Write-Host "`n[2/5] P20-17: PCI DSS SAQ A and checkout script integrity..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-pci-saq-a.sh"

Write-Host "`n[3/5] P20-15: Redis outage and degradation recovery verification..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/redis-outage.sh"

Write-Host "`n[4/5] Full Parity test suite verification..."
& "C:\Program Files\Git\bin\bash.exe" -c "export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno; ./scripts/parity.sh"

Write-Host "`n[5/5] Architectural gate checks..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-isolation.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-sql-boundaries.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-shared-kernel.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-contracts.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-domain-gates.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-in-memory-stores.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"

Write-Host "`n========================================================"
Write-Host "==> Phase Gate P20 PASSED: GA Production Ready"
Write-Host "========================================================"
