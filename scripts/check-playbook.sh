#!/usr/bin/env bash
# P12-01: every module declared migrated actually follows the playbook.
#
# The playbook is only worth writing if it is followed, and "followed" is
# checkable: each step below leaves a mechanical trace in the tree. This checks
# the tree, not the prose.
set -euo pipefail

PLAYBOOK="docs/MIGRATION_PLAYBOOK.md"
DECL="scripts/check-no-in-memory-stores.sh"

fail() { echo "FAIL: $*" >&2; FAILED=1; }
FAILED=0

echo "==> Checking the migration playbook is present and followed..."

[ -f "$PLAYBOOK" ] || { echo "FAIL: $PLAYBOOK is missing." >&2; exit 1; }

# The playbook must cover every step a migration actually needs. These are the
# headings the P12-P14 migrations will be read against.
for section in \
    "Write the migration" \
    "Make the entities persistable" \
    "Rewrite the service" \
    "Convert the tests" \
    "Wire up the call sites" \
    "Declare it migrated" \
    "Run the gates"
do
    grep -qF "$section" "$PLAYBOOK" || fail "$PLAYBOOK has no '$section' step."
done

# Read the migrated list from its single source of truth, so the two scripts
# cannot disagree about which modules are done.
mapfile -t MIGRATED < <(
    sed -n '/^MIGRATED=(/,/^)/p' "$DECL" \
    | grep -oE '"stheno-[a-z-]+"' \
    | tr -d '"'
)

[ "${#MIGRATED[@]}" -gt 0 ] || { echo "FAIL: no migrated modules declared in $DECL." >&2; exit 1; }

for module in "${MIGRATED[@]}"; do
    src="crates/${module#stheno-}"
    src="crates/modules/${module}/src"
    tests="crates/modules/${module}/tests"
    short="${module#stheno-}"

    echo "  -- ${module}"

    # 1. A migration exists for the module's schema.
    #
    # Either a migration is named after the module, or every table the module's entities
    # declare (`const TABLE`) is created by some migration. The second form exists because
    # the identity, catalog-config and platform-ops modules were migrated together in grouped
    # schema migrations (`*_identity_access_schema`, `*_catalog_config_schema`,
    # `*_platform_ops_schema`); a filename match alone can never see those. It is the stronger
    # evidence of the two: it checks each table, not one name.
    if ! ls migrations/ 2>/dev/null | grep -qE "_${short//-/_}_"; then
        mapfile -t tables < <(
            grep -rhoE 'const TABLE: &(.static )?str = "[a-z_.]+"' "$src" 2>/dev/null \
            | grep -oE '"[a-z_.]+"' | tr -d '"' | sort -u
        )
        if [ "${#tables[@]}" -eq 0 ]; then
            fail "${module}: no migration in migrations/ names this module."
        else
            for table in "${tables[@]}"; do
                pattern="create table (if not exists )?\"?${table//./\"?\\.\"?}\"?[ (]"
                # Files passed directly, not piped from cat: under `pipefail`, `grep -q` exiting
                # on the first match kills cat with SIGPIPE and the pipeline reports failure.
                grep -qiE "$pattern" migrations/*/up.sql \
                    || fail "${module}: table ${table} is not created by any migration."
            done
        fi
    fi

    # 2. Entities are bound to tables through the repository trait.
    grep -rq "impl CrudEntity for" "$src" \
        || fail "${module}: no CrudEntity impl; entities are not bound to any table."
    grep -rq "const TABLE" "$src" \
        || fail "${module}: no TABLE constant on its entities."

    # 3. The service holds repositories and takes a pool.
    if [ -f "$src/service.rs" ]; then
        grep -q "PgCrudRepository" "$src/service.rs" \
            || fail "${module}: service.rs does not use PgCrudRepository."
        # Any constructor that takes a DbPool counts: `AuthService::new(pool, jwt_secret)`,
        # `FileService::new(provider, pool)` and search's storage-backed constructor are all
        # pool-backed. What fails is a service with no pool-taking constructor at all.
        grep -qE "pub (async )?fn [a-z_]+\([^)]*: DbPool" "$src/service.rs" \
            || fail "${module}: service constructor does not take a DbPool."
    fi

    # 4. Suffix id matching is gone. It is a full table scan on an indexed
    #    column and only ever existed to paper over the lost id prefix.
    #    Comment lines are excluded: `grep -rn` output is prefixed with
    #    `file:line:`, so a bare `^//` filter never matches the comment itself.
    if grep -rn "ends_with" "$src" 2>/dev/null         | grep -qvE '^[^:]+:[0-9]+:[[:space:]]*(//|/\*|\*)'; then
        fail "${module}: still matches ids with ends_with (ADR-010)."
    fi

    # 5. A test proves rows outlive the service that wrote them. Without this a
    #    module can pass every other check and still be a decoration.
    if [ -d "$tests" ]; then
        grep -rq "outlive" "$tests" \
            || fail "${module}: no test asserts rows outlive the service instance."
        grep -rq "TestDb::migrated" "$tests" \
            || fail "${module}: tests do not run against a migrated database."
    else
        fail "${module}: has no tests directory."
    fi

    # 6. Every test that provisions a database drops it, or a suite run leaves
    #    one abandoned database behind per test.
    for f in "$tests"/*.rs; do
        [ -f "$f" ] || continue
        created=$(grep -c "TestDb::migrated\|TestDb::create" "$f" || true)
        dropped=$(grep -c "\.drop()" "$f" || true)
        if [ "$created" -gt 0 ] && [ "$dropped" -lt 1 ]; then
            fail "${module}: $(basename "$f") creates a database and never drops it."
        fi
    done
done

if [ "$FAILED" -ne 0 ]; then
    echo "Playbook check FAILED." >&2
    exit 1
fi

echo "Playbook check PASSED: ${#MIGRATED[@]} migrated module(s) follow docs/MIGRATION_PLAYBOOK.md."
exit 0
