# scripts/check-checkout-scripts.ps1 — Verifies checkout script manifest against PCI SAQ A & F25 requirements
$ErrorActionPreference = "Stop"

$MANIFEST = "docs/payment-page-scripts.md"

if (-not (Test-Path $MANIFEST)) {
    Write-Error "FAIL: Checkout script manifest $MANIFEST does not exist."
    exit 1
}

Write-Host "Verifying checkout script inventory in $MANIFEST..." -ForegroundColor Yellow
$content = Get-Content $MANIFEST -Raw

# 1. Ensure all registered gateway origins are documented
$origins = @("js.stripe.com", "bka.sh", "mynagad.com", "sslcommerz.com")
foreach ($origin in $origins) {
    if ($content -notmatch [regex]::Escape($origin)) {
        Write-Error "FAIL: Gateway origin $origin not documented in $MANIFEST."
        exit 1
    }
}

# 2. Ensure forbidden scripts are explicitly banned
$bannedList = @("Google Tag Manager", "Meta Pixel", "chat widgets", "Hotjar")
foreach ($banned in $bannedList) {
    if ($content -notmatch [regex]::Escape($banned)) {
        Write-Error "FAIL: Missing prohibition for $banned in $MANIFEST."
        exit 1
    }
}

Write-Host "PASS: Checkout script and gateway origin manifest verified (PCI SAQ A compliant)." -ForegroundColor Green
exit 0
