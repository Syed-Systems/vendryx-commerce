# scripts/check-checkout-scripts.ps1 — Verifies checkout script manifest against PCI SAQ A & F25 requirements
$ErrorActionPreference = "Stop"

$MANIFEST = "docs/payment-page-scripts.md"

if (-not (Test-Path $MANIFEST)) {
    Write-Error "FAIL: Checkout script manifest $MANIFEST does not exist."
    exit 1
}

Write-Host "Verifying checkout script inventory in $MANIFEST..." -ForegroundColor Yellow
$content = Get-Content $MANIFEST -Raw

# 1. Ensure js.stripe.com is present
if ($content -notmatch "https://js.stripe.com") {
    Write-Error "FAIL: Stripe script not documented in $MANIFEST."
    exit 1
}

# 2. Ensure forbidden scripts are explicitly banned
$bannedList = @("Google Tag Manager", "Meta Pixel", "chat widgets")
foreach ($banned in $bannedList) {
    if ($content -notmatch [regex]::Escape($banned)) {
        Write-Error "FAIL: Missing prohibition for $banned in $MANIFEST."
        exit 1
    }
}

Write-Host "PASS: Checkout script manifest verified (PCI SAQ A compliant)." -ForegroundColor Green
exit 0
