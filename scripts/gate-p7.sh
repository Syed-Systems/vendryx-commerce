#!/usr/bin/env bash
# Phase Gate P7 Verification Script (Auth, User, Customer, API Keys, RBAC, Authorization Matrix)
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P7 (Auth, User, Customer, API Keys, RBAC, OAuth/OIDC, Matrix, Parity)..."
echo "========================================="

# 1. Architectural Invariants
echo -e "\n[1/6] Checking Architectural Isolation, DTO Separation & Contracts..."
./scripts/check-module-isolation.sh
./scripts/check-sql-boundaries.sh
./scripts/check-shared-kernel.sh
./scripts/check-no-domain-gates.sh
./scripts/check-module-contracts.sh
./scripts/check-dto-separation.sh

# 2. Core Auth Module (P7-01, P7-02, P7-06 to P7-13)
echo -e "\n[2/6] Running Core Auth Module Tests (Argon2id, JWT, Rotation, MFA, Reset, Lockout, Breached)..."
cargo test -p stheno-auth

# 3. Auth Providers (P7-03 to P7-05)
echo -e "\n[3/6] Running Auth Providers Tests (EmailPass, Google, GitHub, OIDC Clean-Room)..."
cargo test -p stheno-auth-emailpass -p stheno-auth-google -p stheno-auth-github -p stheno-auth-oidc

# 4. Customer, User, API Key & RBAC Modules (P7-14, P7-15, P7-16, P7-18, P7-19)
echo -e "\n[4/6] Running Customer, User, API Key & RBAC Module Tests..."
cargo test -p stheno-customer
cargo test -p stheno-user
cargo test -p stheno-api-key
cargo test -p stheno-rbac

# 5. Scoping Middleware & Authorization Matrix (P7-17, P7-20)
echo -e "\n[5/6] Running Scope Resolution Middleware & Full Authorization Matrix Tests..."
cargo test -p stheno-http --test scope_resolution
cargo test -p stheno-flows --test authz_matrix

# 6. Medusa Auth API Parity & Workspace Lint/Formatting (P7-21, P7-22)
echo -e "\n[6/6] Running Medusa Auth API Parity Suite & Workspace Verification..."
cargo test -p stheno-api-auth
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

echo "========================================="
echo "==> Phase Gate P7 PASSED: 100% COMPLETE & VERIFIED!"
echo "========================================="
