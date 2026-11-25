# scripts/sync-brand-surfaces.ps1
# Synchronizes all customer-facing brand assets, tokens, and titles from
# Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit (single source of truth).
param(
    [switch]$Write,
    [switch]$Check
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $repoRoot

$kitRoot = Join-Path $repoRoot "Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit"
if (-not (Test-Path $kitRoot)) {
    Write-Error "Canonical brand kit not found at $kitRoot"
    exit 1
}

$manifestPath = Join-Path $repoRoot "brand/vendryx/brand.json"
if (-not (Test-Path $manifestPath)) {
    Write-Error "Brand manifest not found at $manifestPath"
    exit 1
}

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$productName = $manifest.productName
$legalName = $manifest.legalName
$canonicalDomain = $manifest.canonicalDomain

Write-Host "==> Canonical brand source: Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit"
Write-Host "==> Brand: $productName ($legalName), domain: $canonicalDomain"

# Canonical asset mapping: relative path inside Vendryx_Brand_Kit -> target paths relative to repo root
$assetMap = @(
    @{ Src = "09_Developer/tokens.css"; Dst = @("brand/vendryx/vendryx-tokens.css") },
    @{ Src = "09_Developer/tokens.json"; Dst = @("brand/vendryx/vendryx-tokens.json") },

    @{ Src = "01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg"; Dst = @("brand/vendryx/vendryx-primary-light.svg", "apps/admin/public/brand/vendryx/vendryx-primary-light.svg", "apps/platform/public/brand/vendryx/vendryx-primary-light.svg", "apps/marketing/public/brand/vendryx/vendryx-primary-light.svg", "apps/marketing/public/branding/vendryx-primary-light.svg") },
    @{ Src = "01_Primary_Logos/vendryx_primary_dark_with_by_syed_systems.svg"; Dst = @("brand/vendryx/vendryx-primary-dark.svg", "apps/admin/public/brand/vendryx/vendryx-primary-dark.svg", "apps/platform/public/brand/vendryx/vendryx-primary-dark.svg", "apps/marketing/public/brand/vendryx/vendryx-primary-dark.svg", "apps/marketing/public/branding/vendryx-primary-dark.svg") },
    @{ Src = "01_Primary_Logos/vendryx_primary_light_no_tagline_1200w.png"; Dst = @("brand/vendryx/vendryx-logo.png", "apps/admin/public/brand/vendryx/vendryx-logo.png", "apps/platform/public/brand/vendryx/vendryx-logo.png", "apps/marketing/public/brand/vendryx/vendryx-logo.png") },
    @{ Src = "01_Primary_Logos/vendryx_primary_dark_no_tagline_1200w.png"; Dst = @("brand/vendryx/vendryx-logo-dark.png", "apps/admin/public/brand/vendryx/vendryx-logo-dark.png", "apps/platform/public/brand/vendryx/vendryx-logo-dark.png", "apps/marketing/public/brand/vendryx/vendryx-logo-dark.png") },
    @{ Src = "01_Primary_Logos/vendryx_primary_light_with_by_syed_systems.svg"; Dst = @("brand/vendryx/vendryx-with-tagline.svg", "apps/marketing/public/brand/vendryx/vendryx-with-tagline.svg") },

    @{ Src = "02_Secondary_Logos/vendryx_stacked_light_with_by_syed_systems.svg"; Dst = @("brand/vendryx/vendryx-stacked-light.svg", "apps/marketing/public/brand/vendryx/vendryx-stacked-light.svg") },
    @{ Src = "02_Secondary_Logos/vendryx_stacked_dark_with_by_syed_systems.svg"; Dst = @("brand/vendryx/vendryx-stacked-dark.svg", "apps/marketing/public/brand/vendryx/vendryx-stacked-dark.svg") },

    @{ Src = "04_Symbol/vendryx_tile_dark.svg"; Dst = @("brand/vendryx/vendryx-mark.svg", "apps/admin/public/brand/vendryx/vendryx-mark.svg", "apps/platform/public/brand/vendryx/vendryx-mark.svg", "apps/marketing/public/brand/vendryx/vendryx-mark.svg", "apps/marketing/public/branding/vendryx-mark-dark.svg") },
    @{ Src = "04_Symbol/vendryx_tile_dark_512w.png"; Dst = @("brand/vendryx/vendryx-mark.png", "apps/admin/public/brand/vendryx/vendryx-mark.png", "apps/platform/public/brand/vendryx/vendryx-mark.png", "apps/marketing/public/brand/vendryx/vendryx-mark.png") },
    @{ Src = "04_Symbol/vendryx_mark_black_transparent.svg"; Dst = @("brand/vendryx/vendryx-black.svg") },
    @{ Src = "04_Symbol/vendryx_mark_white_transparent.svg"; Dst = @("brand/vendryx/vendryx-white.svg") },

    @{ Src = "05_Favicons/favicon.ico"; Dst = @("brand/vendryx/favicon.ico", "apps/admin/public/favicon.ico", "apps/admin/public/brand/vendryx/favicon.ico", "apps/platform/public/favicon.ico", "apps/platform/public/brand/vendryx/favicon.ico", "apps/marketing/public/favicon.ico", "apps/marketing/public/brand/vendryx/favicon.ico") },
    @{ Src = "05_Favicons/favicon.svg"; Dst = @("brand/vendryx/favicon.svg", "apps/admin/public/brand/vendryx/favicon.svg", "apps/platform/public/brand/vendryx/favicon.svg", "apps/marketing/public/brand/vendryx/favicon.svg", "apps/marketing/public/branding/favicon.svg") },
    @{ Src = "05_Favicons/favicon-16x16.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-16x16.png", "apps/platform/public/brand/vendryx/favicon-16x16.png", "apps/marketing/public/brand/vendryx/favicon-16x16.png") },
    @{ Src = "05_Favicons/favicon-32x32.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-32x32.png", "apps/platform/public/brand/vendryx/favicon-32x32.png", "apps/marketing/public/brand/vendryx/favicon-32x32.png") },
    @{ Src = "05_Favicons/favicon-48x48.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-48x48.png", "apps/platform/public/brand/vendryx/favicon-48x48.png", "apps/marketing/public/brand/vendryx/favicon-48x48.png") },
    @{ Src = "05_Favicons/favicon-64x64.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-64x64.png", "apps/platform/public/brand/vendryx/favicon-64x64.png", "apps/marketing/public/brand/vendryx/favicon-64x64.png") },
    @{ Src = "05_Favicons/favicon-96x96.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-96x96.png", "apps/platform/public/brand/vendryx/favicon-96x96.png", "apps/marketing/public/brand/vendryx/favicon-96x96.png") },
    @{ Src = "04_Symbol/vendryx_tile_dark_512w.png"; Dst = @("brand/vendryx/favicon.png", "apps/admin/public/favicon.png", "apps/admin/public/brand/vendryx/favicon.png", "apps/platform/public/favicon.png", "apps/platform/public/brand/vendryx/favicon.png", "apps/marketing/public/favicon.png", "apps/marketing/public/brand/vendryx/favicon.png") },
    @{ Src = "04_Symbol/vendryx_tile_dark_128w.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-128x128.png", "apps/platform/public/brand/vendryx/favicon-128x128.png", "apps/marketing/public/brand/vendryx/favicon-128x128.png") },
    @{ Src = "04_Symbol/vendryx_tile_dark_256w.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-256x256.png", "apps/platform/public/brand/vendryx/favicon-256x256.png", "apps/marketing/public/brand/vendryx/favicon-256x256.png") },
    @{ Src = "04_Symbol/vendryx_tile_dark_512w.png"; Dst = @("apps/admin/public/brand/vendryx/favicon-512x512.png", "apps/platform/public/brand/vendryx/favicon-512x512.png", "apps/marketing/public/brand/vendryx/favicon-512x512.png") },

    @{ Src = "06_App_PWA/app-icon-dark-180.png"; Dst = @("brand/vendryx/apple-touch-icon.png", "apps/admin/public/apple-touch-icon.png", "apps/admin/public/brand/vendryx/apple-touch-icon.png", "apps/platform/public/apple-touch-icon.png", "apps/platform/public/brand/vendryx/apple-touch-icon.png", "apps/marketing/public/apple-touch-icon.png", "apps/marketing/public/brand/vendryx/apple-touch-icon.png", "apps/marketing/public/branding/apple-touch-icon.png") },
    @{ Src = "06_App_PWA/app-icon-dark-192.png"; Dst = @("brand/vendryx/app-icon-192.png", "apps/admin/public/brand/vendryx/app-icon-192.png", "apps/platform/public/brand/vendryx/app-icon-192.png", "apps/marketing/public/brand/vendryx/app-icon-192.png") },
    @{ Src = "06_App_PWA/app-icon-dark-512.png"; Dst = @("brand/vendryx/app-icon-512.png", "apps/admin/public/brand/vendryx/app-icon-512.png", "apps/platform/public/brand/vendryx/app-icon-512.png", "apps/marketing/public/brand/vendryx/app-icon-512.png") },
    @{ Src = "06_App_PWA/maskable-icon-512.png"; Dst = @("brand/vendryx/maskable-icon-512.png", "apps/admin/public/brand/vendryx/maskable-icon-512.png", "apps/platform/public/brand/vendryx/maskable-icon-512.png", "apps/marketing/public/brand/vendryx/maskable-icon-512.png") },

    @{ Src = "07_Social_Marketing/open-graph.png"; Dst = @("apps/marketing/public/brand/vendryx/open-graph-1200x630.png", "apps/marketing/public/branding/open-graph.png") },
    @{ Src = "07_Social_Marketing/open-graph.svg"; Dst = @("apps/marketing/public/brand/vendryx/open-graph-1200x630.svg") }
)

$drift = $false

foreach ($entry in $assetMap) {
    $srcPath = Join-Path $kitRoot $entry.Src
    if (-not (Test-Path $srcPath)) {
        Write-Error "Missing canonical asset: $srcPath"
        exit 1
    }
    $srcHash = (Get-FileHash -Path $srcPath -Algorithm SHA256).Hash

    foreach ($relDst in $entry.Dst) {
        $dstPath = Join-Path $repoRoot $relDst
        $dstDir = Split-Path -Parent $dstPath
        $matches = (Test-Path $dstPath) -and ((Get-FileHash -Path $dstPath -Algorithm SHA256).Hash -eq $srcHash)

        if (-not $matches) {
            $drift = $true
            if ($Write) {
                if (-not (Test-Path $dstDir)) {
                    New-Item -ItemType Directory -Path $dstDir -Force | Out-Null
                }
                Copy-Item -Path $srcPath -Destination $dstPath -Force
                Write-Host "  Synced $relDst <- $($entry.Src)"
            } else {
                Write-Host "  Drift in $relDst (does not match $($entry.Src))"
            }
        }
    }
}

# Sync customer-facing HTML titles
$targets = @(
    @{ Path = "apps/marketing/index.html"; Desired = "$productName | Your Online Store, Your Brand" },
    @{ Path = "apps/admin/index.html"; Desired = "$productName Dashboard" },
    @{ Path = "apps/platform/index.html"; Desired = "$productName Platform" }
)

foreach ($target in $targets) {
    $fullPath = Join-Path $repoRoot $target.Path
    if (-not (Test-Path $fullPath)) { continue }

    $content = Get-Content $fullPath -Raw
    if ($content -match "<title>(.*?)</title>") {
        $currentTitle = $matches[1]
        if ($currentTitle -ne $target.Desired) {
            $drift = $true
            if ($Write) {
                $updated = $content -replace "<title>.*?</title>", "<title>$($target.Desired)</title>"
                Set-Content -Path $fullPath -Value $updated -NoNewline
                Write-Host "  Updated title in $($target.Path) -> $($target.Desired)"
            } else {
                Write-Host "  Drift in $($target.Path): title is '$currentTitle', expected '$($target.Desired)'"
            }
        }
    }
}

if ($Check) {
    if ($drift) {
        Write-Host "FAIL: Brand surface drift detected. Run ./scripts/sync-brand-surfaces.ps1 -Write to synchronize." -ForegroundColor Red
        exit 1
    } else {
        Write-Host "Brand surfaces check PASSED: All targets match Vendryx_Approved_Complete_Brand_Package/Vendryx_Brand_Kit" -ForegroundColor Green
        exit 0
    }
} elseif ($Write) {
    Write-Host "Brand surfaces synchronized successfully." -ForegroundColor Green
    exit 0
} else {
    Write-Host "Usage: ./scripts/sync-brand-surfaces.ps1 -Write | -Check"
    exit 0
}
