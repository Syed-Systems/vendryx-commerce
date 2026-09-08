# scripts/check-no-mock-storage.ps1
# Enforces ADR-029: No mock storage or simulated persistence in merchant admin production routes.
param()

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

Write-Host "==> Checking apps/admin/src/routes for forbidden mock storage..."

$routesDir = Join-Path $repoRoot "apps/admin/src/routes"
if (-not (Test-Path $routesDir)) {
    Write-Host "Directory $routesDir does not exist; skipping check."
    exit 0
}

# Scan all ts/tsx files in apps/admin/src/routes, excluding __tests__ directories
$files = Get-ChildItem -Path $routesDir -Recurse -Include *.ts, *.tsx | Where-Object {
    $_.FullName -notmatch "[\\/]__tests__[\\/]" -and $_.FullName -notmatch "\.spec\.tsx?$" -and $_.FullName -notmatch "\.test\.tsx?$"
}

$failed = $false

foreach ($file in $files) {
    $relPath = $file.FullName.Substring($repoRoot.Path.Length + 1)
    $lines = Get-Content $file.FullName
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        # Disallow localStorage and sessionStorage in production routes
        if ($line -match "\b(localStorage|sessionStorage)\b") {
            Write-Host "FAIL: Prohibited browser storage in ${relPath}:${lineNum}: $line" -ForegroundColor Red
            $failed = $true
        }
    }
}

if ($failed) {
    Write-Host "FAIL: Found prohibited mock storage in apps/admin routes. Back all state with genuine backend endpoints per ADR-029." -ForegroundColor Red
    exit 1
} else {
    Write-Host "Mock storage check PASSED: apps/admin routes contain zero mock storage." -ForegroundColor Green
    exit 0
}
