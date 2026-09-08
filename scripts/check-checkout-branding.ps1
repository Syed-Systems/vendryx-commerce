# scripts/check-checkout-branding.ps1
# P26-08a: Checkout branding compliance and script isolation.

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "==> Verifying checkout branding compliance (P26-08a)..." -ForegroundColor Cyan

$ScanPaths = @(
    "apps/storefront/src/app/[countryCode]/(checkout)",
    "apps/storefront/src/modules/checkout",
    "apps/storefront/packages/storefront-core/src/modules/checkout"
)

$ForbiddenRegex = '(fonts\.googleapis\.com|fonts\.gstatic\.com|use\.typekit\.net|cdn\.jsdelivr\.net|unpkg\.com)'

foreach ($Path in $ScanPaths) {
    if (Test-Path $Path) {
        $Matches = Get-ChildItem -Path $Path -Recurse -File | Select-String -Pattern $ForbiddenRegex
        if ($Matches) {
            Write-Error "FAIL: Third-party CDN reference found in checkout path $Path: $Matches"
            exit 1
        }
    }
}

$LayoutFile = "apps/storefront/packages/storefront-core/src/app/layout.tsx"
if (-not (Test-Path $LayoutFile) -or (Get-Content $LayoutFile -Raw) -notmatch "stheno-brand-tokens") {
    Write-Error "FAIL: RootLayout does not inject stheno-brand-tokens into <head>."
    exit 1
}

Write-Host "Checkout branding check PASSED: Checkout routes apply brand tokens and obey script isolation." -ForegroundColor Green
exit 0
