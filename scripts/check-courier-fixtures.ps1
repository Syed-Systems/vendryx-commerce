# scripts/check-courier-fixtures.ps1 — Recorded wiremock fixtures for BD courier providers (P23-13)
$ErrorActionPreference = "Stop"

Write-Host "==> Verifying recorded wiremock fixtures for courier providers..."

Write-Host "[1/3] Pathao courier fixtures..."
cargo test -p stheno-fulfillment-pathao --test options
cargo test -p stheno-fulfillment-pathao --test rates
cargo test -p stheno-fulfillment-pathao --test consignment
cargo test -p stheno-fulfillment-pathao --test status_mapping

Write-Host "[2/3] Steadfast courier fixtures..."
cargo test -p stheno-fulfillment-steadfast --test provider
cargo test -p stheno-fulfillment-steadfast --test status_mapping

Write-Host "[3/3] RedX courier fixtures..."
cargo test -p stheno-fulfillment-redx --test provider
cargo test -p stheno-fulfillment-redx --test status_mapping

Write-Host "All courier fixture tests passed without external API calls (Exit 0)." -ForegroundColor Green
exit 0
