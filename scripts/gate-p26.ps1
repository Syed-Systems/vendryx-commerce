# Phase Gate P26: Branding tier & runtime tokens (PowerShell wrapper)
$ErrorActionPreference = "Stop"

if (-not $env:DATABASE_URL) {
    $env:DATABASE_URL = "postgres://stheno:stheno@localhost:5432/stheno"
}
if (-not $env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR = "C:\Users\Mahi\rust-target"
}

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P26: Branding tier & runtime tokens"
Write-Host "========================================================"

Write-Host "`n[1/10] P26-01: Confirm ADR-012 tenancy audit is green..."
cargo test -p stheno-db --test rls_audit
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[2/10] P26-02: Tenant branding schema migration & drift check..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test -p stheno-db --test rls_audit
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[3/10] P26-03: Store API branding endpoint & defaults..."
cargo test -p stheno-api-store --test branding_endpoint
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[4/10] P26-04: Curated self-hosted font licences check..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-font-licences.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[5/10] P26-05: Storefront core token resolution & CSS properties..."
npm.cmd --prefix apps/storefront run test -- branding-tokens
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[6/10] P26-06: Logo & favicon type and dimension validation..."
cargo test -p stheno-file --test branding_asset_validation
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[7/10] P26-07: Admin branding UI with live preview..."
npm.cmd --prefix apps/admin run test -- branding
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[8/10] P26-08: Multi-tenant isolated branding renders..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-multi-tenant-branding.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[9/10] P26-08a: Checkout branding & clean script manifest..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-checkout-branding.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-checkout-scripts.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n[10/10] P26-09: Lighthouse audit & 0 round-trip render-path branding..."
npm.cmd --prefix apps/storefront run test:lighthouse
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host "`n========================================================"
Write-Host "==> PASS: Phase Gate P26"
Write-Host "========================================================"
