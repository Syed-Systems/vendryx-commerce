# scripts/check-provider-pci.ps1 — Mechanical PCI SAQ A enforcement on payment providers (P21-13)
$ErrorActionPreference = "Stop"

Write-Host "==> Checking payment provider PCI SAQ A compliance..."

$forbiddenPatterns = "card_number|credit_card_pan|\bpan\b.*String|\bcvv\b.*String|\bcvc\b.*String|card_cvv|card_pan|track1|track2"

# 1. Selftest: Ensure the check catches a deliberately non-compliant fixture
$fixtureNonCompliant = "struct MaliciousDirectCardProvider { pan: String, cvv: String }"
if ($fixtureNonCompliant -notmatch $forbiddenPatterns) {
    Write-Error "ERROR: Selftest failed! check-provider-pci did not detect non-compliant card-data pattern."
    exit 1
}
Write-Host "✓ Selftest passed: Deliberately non-compliant fixture detected and refused."

# 2. Audit all payment provider crates
$violations = Get-ChildItem -Path "crates/providers/stheno-payment-*/src/*.rs" | Select-String -Pattern $forbiddenPatterns

if ($violations) {
    Write-Error "ERROR: Forbidden cardholder data fields detected in provider crates (PCI SAQ A violation)!"
    $violations | ForEach-Object { Write-Error $_ }
    exit 1
}

Write-Host "Provider PCI check PASSED: Zero cardholder data flows across all payment provider crates."
exit 0
