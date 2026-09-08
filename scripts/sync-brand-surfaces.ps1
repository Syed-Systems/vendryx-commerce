# scripts/sync-brand-surfaces.ps1
# Synchronizes customer-facing brand surfaces with brand/vendryx/brand.json
param(
    [switch]$Write,
    [switch]$Check
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$manifestPath = Join-Path $repoRoot "brand/vendryx/brand.json"
if (-not (Test-Path $manifestPath)) {
    Write-Error "Brand manifest not found at $manifestPath"
    exit 1
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$productName = $manifest.productName
$legalName = $manifest.legalName
$canonicalDomain = $manifest.canonicalDomain

Write-Host "==> Operating on brand: $productName ($legalName), domain: $canonicalDomain"

# Target files that are explicitly customer-facing
$targets = @(
    "apps/marketing/index.html",
    "apps/admin/index.html",
    "apps/platform/index.html"
)

$drift = $false

foreach ($relPath in $targets) {
    $fullPath = Join-Path $repoRoot $relPath
    if (-not (Test-Path $fullPath)) {
        continue
    }

    $content = Get-Content $fullPath -Raw
    $original = $content

    # Check/replace title
    if ($content -match "<title>(.*?)</title>") {
        $currentTitle = $matches[1]
        $desiredTitle = "$productName"
        if ($relPath -like "*platform*") {
            $desiredTitle = "$productName Platform"
        } elseif ($relPath -like "*admin*") {
            $desiredTitle = "$productName Dashboard"
        }

        if ($currentTitle -ne $desiredTitle) {
            $drift = $true
            if ($Write) {
                $content = $content -replace "<title>.*?</title>", "<title>$desiredTitle</title>"
                Write-Host "  Updated title in $relPath -> $desiredTitle"
            } else {
                Write-Host "  Drift in ${relPath}: title is '$currentTitle', expected '$desiredTitle'"
            }
        }
    }

    if ($Write -and ($content -ne $original)) {
        Set-Content -Path $fullPath -Value $content -NoNewline
    }
}

if ($Check) {
    if ($drift) {
        Write-Host "FAIL: Brand surface drift detected. Run ./scripts/sync-brand-surfaces.ps1 -Write to synchronize." -ForegroundColor Red
        exit 1
    } else {
        Write-Host "Brand surfaces check PASSED: All customer-facing targets match $manifestPath" -ForegroundColor Green
        exit 0
    }
} elseif ($Write) {
    Write-Host "Brand surfaces synchronized successfully." -ForegroundColor Green
    exit 0
} else {
    Write-Host "Usage: ./scripts/sync-brand-surfaces.ps1 -Write | -Check"
    exit 0
}
