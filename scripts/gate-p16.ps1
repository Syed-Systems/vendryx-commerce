# Phase Gate P16: Reporting Module & Analytics Rollups
$ErrorActionPreference = "Stop"

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P16: Reporting Module"
Write-Host "========================================================"

Write-Host "`n[1/6] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n[2/6] P16-01: stheno-reporting unit tests..."
cargo test -p stheno-reporting

Write-Host "`n[3/6] P16-02 to P16-11: Rollup schemas, refresh, invalidation, aggregates, tier horizons..."
cargo test -p stheno-reporting --test rollup_schema
cargo test -p stheno-reporting --test rollup_incremental
cargo test -p stheno-reporting --test rollup_invalidation
cargo test -p stheno-reporting --test revenue_over_time
cargo test -p stheno-reporting --test orders_over_time
cargo test -p stheno-reporting --test top_products
cargo test -p stheno-reporting --test average_order_value
cargo test -p stheno-reporting --test window_gate
cargo test -p stheno-reporting --test free_tier_horizon
cargo test -p stheno-reporting --test pro_tier_horizon

Write-Host "`n[4/6] P16-12: Medusa envelope shape for reports..."
cargo test -p stheno-api-admin --test reports_parity

Write-Host "`n[5/6] P16-13: Report CSV export with UTF-8 BOM..."
cargo test -p stheno-reporting --test report_csv_export

Write-Host "`n[6/6] Architectural gate checks..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-isolation.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-sql-boundaries.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-shared-kernel.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-contracts.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-domain-gates.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-in-memory-stores.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"

Write-Host "`n========================================================"
Write-Host "==> Phase Gate P16 PASSED"
Write-Host "========================================================"
