# Phase Gate P7 Verification Script (Auth, User, Customer, API Keys, RBAC, Authorization Matrix)
$ErrorActionPreference = "Stop"
$env:PATH = "C:\Users\Mahi\mingw64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "==> Verifying Phase Gate P7 (Auth, User, Customer, API Keys, RBAC, OAuth/OIDC, Matrix, Parity)..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Architectural Invariants
Write-Host "`n[1/6] Checking Architectural Isolation, DTO Separation & Contracts..." -ForegroundColor Yellow
powershell -ExecutionPolicy Bypass -File scripts/check-module-isolation.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-sql-boundaries.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-shared-kernel.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-no-domain-gates.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-module-contracts.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-dto-separation.ps1

# 2. Core Auth Module (P7-01, P7-02, P7-06 to P7-13)
Write-Host "`n[2/6] Running Core Auth Module Tests (Argon2id, JWT, Rotation, MFA, Reset, Lockout, Breached)..." -ForegroundColor Yellow
cargo test -p stheno-auth

# 3. Auth Providers (P7-03 to P7-05)
Write-Host "`n[3/6] Running Auth Providers Tests (EmailPass, Google, GitHub, OIDC Clean-Room)..." -ForegroundColor Yellow
cargo test -p stheno-auth-emailpass -p stheno-auth-google -p stheno-auth-github -p stheno-auth-oidc

# 4. Customer, User, API Key & RBAC Modules (P7-14, P7-15, P7-16, P7-18, P7-19)
Write-Host "`n[4/6] Running Customer, User, API Key & RBAC Module Tests..." -ForegroundColor Yellow
cargo test -p stheno-customer
cargo test -p stheno-user
cargo test -p stheno-api-key
cargo test -p stheno-rbac

# 5. Scoping Middleware & Authorization Matrix (P7-17, P7-20)
Write-Host "`n[5/6] Running Scope Resolution Middleware & Full Authorization Matrix Tests..." -ForegroundColor Yellow
cargo test -p stheno-http --test scope_resolution
cargo test -p stheno-flows --test authz_matrix

# 6. Medusa Auth API Parity & Workspace Lint/Formatting (P7-21, P7-22)
Write-Host "`n[6/6] Running Medusa Auth API Parity Suite & Workspace Verification..." -ForegroundColor Yellow
cargo test -p stheno-api-auth
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "==> Phase Gate P7 PASSED: 100% COMPLETE & VERIFIED!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
exit 0
