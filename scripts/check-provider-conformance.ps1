# scripts/check-provider-conformance.ps1 — Provider conformance and capability honesty suite (P21-08..P21-10)
$ErrorActionPreference = "Stop"

Write-Host "==> Running payment provider conformance suite..."

cargo test -p stheno-payment --test conformance_suite -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test -p stheno-payment --test conformance_capabilities -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test -p stheno-payment --test capabilities_redirect -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test -p stheno-payment --test redirect_contract -- --nocapture
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "Provider conformance check PASSED: all providers satisfy lifecycle, idempotency, and capability honesty."
exit 0
