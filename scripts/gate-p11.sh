#!/usr/bin/env bash
# Phase Gate P11: Persistence foundations.
#
# Runs every P11 exit command in dependency order. Each one either measures
# something against a real Postgres or fails. Nothing here asserts a number that
# was not observed.
#
# Per rule L3, tasks that are not done are listed and the gate FAILS. A phase
# gate that passes with tasks outstanding is worse than no gate.
set -euo pipefail

echo "========================================="
echo "==> Verifying Phase Gate P11: Persistence foundations"
echo "========================================="

if [ -z "${DATABASE_URL:-}" ]; then
    echo "FAIL: DATABASE_URL is unset. Every assertion below needs a real database." >&2
    echo "      docker compose up -d postgres" >&2
    echo "      export DATABASE_URL=postgres://stheno:stheno@localhost:5432/stheno" >&2
    exit 1
fi

# Space first: a build that dies on a full disk reports a linker error, and
# that reads as a code defect rather than as the environment problem it is.
./scripts/check-disk-space.sh

echo -e "\n[1/9] Formatting and lints..."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

echo -e "\n[2/9] P11-01 CI Postgres, P11-02 dashboard upstream pin..."
./scripts/check-ci-postgres.sh
./scripts/check-upstream-pin.sh

echo -e "\n[3/9] P11-04 Id<T> wire representation..."
cargo test -p stheno-types --test id_roundtrip

echo -e "\n[4/9] P11-03 boot pool, P11-05 migration runner, P11-06 schema drift..."
cargo test -p stheno-server --test boot_pool
cargo test -p stheno-db --test migration_runner
./scripts/check-schema-drift.sh

echo -e "\n[5/9] P11-07, P11-08 PgCrudRepository..."
cargo test -p stheno-db --test pg_crud_basic
cargo test -p stheno-db --test pg_crud_lifecycle

echo -e "\n[6/9] P11-09 to P11-12 FilterSpec to SQL..."
cargo test -p stheno-db --test filter_sql_scalar
cargo test -p stheno-db --test filter_sql_set
cargo test -p stheno-db --test filter_sql_paging
cargo test -p stheno-db --test filter_hard_cases

echo -e "\n[7/9] P11-13 to P11-15 transactions and concurrency..."
cargo test -p stheno-db --test transaction_nesting
cargo test -p stheno-db --test serializable_retry
cargo test -p stheno-db --test concurrent_writers

echo -e "\n[8/9] P11-16 test harness, P11-17 region, P11-18 currency, P11-21 no in-memory stores..."
cargo test -p stheno-server --test test_harness_isolation
cargo test -p stheno-region
cargo test -p stheno-currency
./scripts/check-no-in-memory-stores.sh

echo -e "\n[9/9] P11-19, P11-20 product migration and P11-22 restart durability..."
OUTSTANDING=0
for check in \
    "P11-19|cargo test -p stheno-product --test persistence" \
    "P11-20|cargo test -p stheno-product --test taxonomy_persistence" \
    "P11-22|./scripts/restart-durability.sh"
do
    id="${check%%|*}"
    cmd="${check#*|}"
    if ! eval "$cmd" >/dev/null 2>&1; then
        echo "  OUTSTANDING: $id -- \`$cmd\` does not pass." >&2
        OUTSTANDING=$((OUTSTANDING + 1))
    else
        echo "  $id passes."
    fi
done

if [ "$OUTSTANDING" -ne 0 ]; then
    echo "=========================================" >&2
    echo "==> Phase Gate P11 FAILED: $OUTSTANDING task(s) outstanding." >&2
    echo "    The product module is still on RwLock<HashMap>, so the phase's" >&2
    echo "    headline assertion -- boot, create a product, restart, retrieve" >&2
    echo "    the same id -- cannot yet be made." >&2
    echo "=========================================" >&2
    exit 1
fi

echo "========================================="
echo "==> Phase Gate P11 PASSED"
echo "========================================="
