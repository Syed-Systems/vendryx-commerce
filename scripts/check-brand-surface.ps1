# scripts/check-brand-surface.ps1
# Verifies customer-facing brand surfaces display Vendryx
param()

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

Write-Host "==> Checking customer-facing brand surfaces for legacy public names..."

$publicFiles = @(
    "apps/marketing/index.html",
    "apps/admin/index.html",
    "apps/platform/index.html",
    "README.md"
)

$allowlistRegex = "(@stheno/|STHENO_|stheno-|stheno_|X-Stheno-|internal `stheno|engine codename `stheno|`stheno-\*`|codename `stheno`|formerly Stheno|powers Stheno|Stheno Commerce CLI|stheno/|postgres://stheno)"

$failed = $false

foreach ($relPath in $publicFiles) {
    $fullPath = Join-Path $repoRoot $relPath
    if (-not (Test-Path $fullPath)) { continue }

    $lines = Get-Content $fullPath
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        if ($line -match "\b(stheno|lokzon)\b") {
            if ($line -notmatch $allowlistRegex) {
                Write-Host "FAIL: Unallowlisted legacy brand match in ${relPath}:${lineNum}: $line" -ForegroundColor Red
                $failed = $true
            }
        }
    }
}

if (Test-Path "apps/marketing/src/App.tsx") {
    $lines = Get-Content "apps/marketing/src/App.tsx"
    $lineNum = 0
    foreach ($line in $lines) {
        $lineNum++
        if ($line -match "\b(lokzon)\b") {
            Write-Host "FAIL: Unallowlisted reference in apps/marketing/src/App.tsx:${lineNum}: $line" -ForegroundColor Red
            $failed = $true
        }
    }
}

if ($failed) {
    Write-Host "FAIL: Brand surface check failed." -ForegroundColor Red
    exit 1
} else {
    Write-Host "Brand surface check PASSED: Customer-facing surfaces are clean." -ForegroundColor Green
    exit 0
}
