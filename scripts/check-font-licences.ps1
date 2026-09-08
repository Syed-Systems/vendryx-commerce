# scripts/check-font-licences.ps1
# P26-04: Curated font set licences and self-hosted enforcement.

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host "==> Checking curated font set licences (P26-04)..." -ForegroundColor Cyan

$FontsDoc = "docs/FONTS.md"
if (-not (Test-Path $FontsDoc)) {
    Write-Error "FAIL: $FontsDoc does not exist."
    exit 1
}

$Content = Get-Content $FontsDoc -Raw
$RequiredFonts = @("inter", "geist", "noto_sans_bengali", "playfair", "plus_jakarta_sans", "roboto", "cinzel")

foreach ($Font in $RequiredFonts) {
    if ($Content -notmatch "\`$Font\`") {
        Write-Error "FAIL: Required font key '$Font' is missing from $FontsDoc"
        exit 1
    }
}

# Ensure every listed font has an approved open licence
$Lines = Get-Content $FontsDoc | Where-Object { $_ -match '\|\s*`[a-z_]+`\s*\|' }
foreach ($Line in $Lines) {
    if ($Line -notmatch '(SIL Open Font License 1\.1|Apache License 2\.0|MIT)') {
        Write-Error "FAIL: Found unapproved or missing font licence in $FontsDoc: $Line"
        exit 1
    }
}

# Ensure checkout routes never reference third-party font CDNs
$CheckoutDir = "apps/storefront/src/checkout"
if (Test-Path $CheckoutDir) {
    $CdnLeaks = Get-ChildItem -Path $CheckoutDir -Recurse -File | Select-String -Pattern '(fonts\.googleapis\.com|fonts\.gstatic\.com|use\.typekit\.net)'
    if ($CdnLeaks) {
        Write-Error "FAIL: Third-party font CDN reference found on checkout surface: $CdnLeaks"
        exit 1
    }
}

Write-Host "Curated font set licences PASSED: All keys registered, licence-cleared, and self-hosted." -ForegroundColor Green
exit 0
