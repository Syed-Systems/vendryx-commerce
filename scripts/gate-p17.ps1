# Phase Gate P17: Admin API Completion & Full Parity Suite
$ErrorActionPreference = "Stop"

Write-Host "========================================================"
Write-Host "==> Verifying Phase Gate P17: Admin API Completion"
Write-Host "========================================================"

Write-Host "`n[1/5] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "`n[2/5] P17-01 to P17-30: Parity test suite (store and admin)..."
& "C:\Program Files\Git\bin\bash.exe" -c "export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno; ./scripts/parity.sh"

Write-Host "`n[3/5] P17-02: Admin route coverage test..."
cargo test -p stheno-api-admin --test route_coverage

Write-Host "`n[4/5] P17-31: Router tier matrix and conditional mounting..."
cargo test -p stheno-api-router --test tier_matrix

Write-Host "`n[5/5] Architectural gate checks..."
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-isolation.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-sql-boundaries.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-shared-kernel.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-module-contracts.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-domain-gates.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-no-in-memory-stores.sh"
& "C:\Program Files\Git\bin\bash.exe" -c "./scripts/check-schema-drift.sh"

Write-Host "`n========================================================"
Write-Host "==> Phase Gate P17 PASSED"
Write-Host "========================================================"
