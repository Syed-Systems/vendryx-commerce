#!/usr/bin/env bash
# P11-06: the schema the binary expects matches the schema the database has.
#
# Phase 1's version grepped `git status` for modified schema.rs files, which
# measured nothing about any database. This one runs the real binary's
# `--check-migrations` against a real Postgres and asserts on its exit code,
# including against a deliberately drifted schema.
set -euo pipefail

echo "==> Checking schema drift..."

if [ -z "${DATABASE_URL:-}" ]; then
    echo "FAIL: DATABASE_URL is unset; schema drift cannot be checked without a database." >&2
    exit 1
fi

# In CI, an uncommitted schema.rs means generated output was left behind by the
# commit that changed a migration. Locally it just means work is in progress, so
# the assertion is scoped to CI rather than dropped -- it is meaningless where a
# dirty tree is the normal state, and load-bearing where it is not.
if [ -n "${CI:-}" ] && git status --porcelain | grep -E "schema\.rs" > /dev/null 2>&1; then
    echo "ERROR: Uncommitted schema.rs changes detected." >&2
    git diff -- "crates/modules/*/src/schema.rs"
    exit 1
fi

# Every migration must be reversible, or drift can be detected but not undone.
cargo test -p stheno-db --test migration_runner

# Drives the binary: migrated schema exits 0, drifted schema exits non-zero.
cargo test -p stheno-server --test check_migrations

echo "Schema drift check PASSED: --check-migrations agrees with the embedded migrations."
exit 0
