Write-Host "==> Running negative boundary check harness (P0-25)..." -ForegroundColor Cyan

$tempFile = "crates/modules/stheno-cart/src/temp_violation.rs"
$tempSqlDir = "crates/modules/stheno-cart/migrations"
$tempSql = "$tempSqlDir/temp_violation.sql"
$kernelViolation = "crates/core/stheno-types/src/temp_kernel.rs"

function Cleanup {
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $tempSql -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $kernelViolation -Force -ErrorAction SilentlyContinue
}

try {
    # 1. Test check-sql-boundaries fails on cross-schema SQL
    Write-Host "Testing check-sql-boundaries on planted violation..."
    Set-Content -Path $tempFile -Value 'pub fn bad() { let _ = "sql_query SELECT * FROM pricing.price"; }'
    & ./scripts/check-sql-boundaries.ps1 *>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "FAILED: check-sql-boundaries did not catch planted violation!" -ForegroundColor Red
        exit 1
    }
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
    Write-Host "  Passed: check-sql-boundaries properly failed on violation." -ForegroundColor Green

    # 2. Test check-shared-kernel fails on domain type in kernel
    Write-Host "Testing check-shared-kernel on planted violation..."
    Set-Content -Path $kernelViolation -Value "pub struct Product { pub id: String }"
    & ./scripts/check-shared-kernel.ps1 *>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "FAILED: check-shared-kernel did not catch planted violation!" -ForegroundColor Red
        exit 1
    }
    Remove-Item -Path $kernelViolation -Force -ErrorAction SilentlyContinue
    Write-Host "  Passed: check-shared-kernel properly failed on violation." -ForegroundColor Green

    # 3. Test check-no-domain-gates fails on Entitlements in module
    Write-Host "Testing check-no-domain-gates on planted violation..."
    Set-Content -Path $tempFile -Value "pub fn check_tier(e: &Entitlements) {}"
    & ./scripts/check-no-domain-gates.ps1 *>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "FAILED: check-no-domain-gates did not catch planted violation!" -ForegroundColor Red
        exit 1
    }
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
    Write-Host "  Passed: check-no-domain-gates properly failed on violation." -ForegroundColor Green

    # 4. Test check-migration-phases fails on unphased DROP COLUMN
    Write-Host "Testing check-migration-phases on planted violation..."
    if (-not (Test-Path $tempSqlDir)) { New-Item -ItemType Directory -Path $tempSqlDir -Force | Out-Null }
    Set-Content -Path $tempSql -Value "ALTER TABLE cart DROP COLUMN customer_id;"
    & ./scripts/check-migration-phases.ps1 *>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "FAILED: check-migration-phases did not catch planted violation!" -ForegroundColor Red
        exit 1
    }
    Remove-Item -Path $tempSql -Force -ErrorAction SilentlyContinue
    Write-Host "  Passed: check-migration-phases properly failed on violation." -ForegroundColor Green

    Write-Host "All negative boundary tests PASSED: All boundary enforcement scripts are verified active and strict." -ForegroundColor Green
    exit 0
}
finally {
    Cleanup
}
