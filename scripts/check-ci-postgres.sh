#!/usr/bin/env bash
# P11-01: CI runs tests against a real Postgres, and local setup is reproducible.
#
# Every assertion below reads a file and checks its contents. Nothing is assumed.
set -euo pipefail

WORKFLOW=".github/workflows/ci.yml"
COMPOSE="docker-compose.yml"
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "==> Checking CI Postgres configuration..."

[ -f "$WORKFLOW" ] || fail "$WORKFLOW is missing; CI does not exist."
[ -f "$COMPOSE" ]  || fail "$COMPOSE is missing; local setup is undocumented."

# The test job must declare a Postgres service container.
grep -qE '^\s+image:\s*postgres:' "$WORKFLOW" \
  || fail "$WORKFLOW declares no postgres service image."

# DATABASE_URL must be exported to the test job, or every test silently
# falls back to in-memory and proves nothing (PHASE2_TASKS.md 1.1).
grep -qE '^\s+DATABASE_URL:\s*postgres://' "$WORKFLOW" \
  || fail "$WORKFLOW does not set DATABASE_URL for the test job."

# A health check, or the job races the database on cold start.
grep -qE 'pg_isready' "$WORKFLOW" \
  || fail "$WORKFLOW postgres service has no pg_isready health check."

grep -qE 'affected-cargo-gate\.py .*--gate test' "$WORKFLOW" \
  || fail "$WORKFLOW does not run the affected-package Rust test gate."
grep -qF 'command.append("--workspace")' scripts/affected-cargo-gate.py \
  || fail "The affected-package gate does not fall back to the complete Cargo workspace."
grep -q -- '--force-full' "$WORKFLOW" \
  || fail "$WORKFLOW does not provide an explicit full-workspace Rust gate."

# Nightly invokes repository scripts through bash because GitHub checkout preserves
# their non-executable 100644 mode; direct ./script.sh calls fail with exit 126.
NIGHTLY=".github/workflows/nightly.yml"
[ -f "$NIGHTLY" ] || fail "$NIGHTLY is missing."
grep -qF 'run: bash scripts/dev-tenants.sh up' "$NIGHTLY" \
  || fail "$NIGHTLY must invoke dev-tenants.sh through bash."
grep -qF 'run: bash scripts/restore-drill.sh' "$NIGHTLY" \
  || fail "$NIGHTLY must invoke restore-drill.sh through bash."
grep -qF 'run: bash scripts/dev-tenants.sh down || true' "$NIGHTLY" \
  || fail "$NIGHTLY must invoke dev-tenants.sh cleanup through bash."

# Local compose must offer the same major version CI runs, so a developer
# cannot pass locally against a Postgres that CI does not have.
ci_pg=$(grep -oE 'postgres:[0-9]+' "$WORKFLOW" | head -1)
local_pg=$(grep -oE 'postgres:[0-9]+' "$COMPOSE" | head -1)
[ -n "$ci_pg" ] && [ "$ci_pg" = "$local_pg" ] \
  || fail "Postgres major version differs: CI has '$ci_pg', compose has '$local_pg'."

grep -q 'DATABASE_URL' README.md \
  || fail "README.md does not document DATABASE_URL for local setup."

echo "CI Postgres check PASSED: $ci_pg in CI and compose, DATABASE_URL set, health check present."
exit 0
