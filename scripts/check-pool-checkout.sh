#!/usr/bin/env bash
# ADR-012: every database connection must be checked out through a tenant-scoping helper.
#
# `PgCrudRepository::with_conn` and `stheno_db::tenant::with_tenant_conn` are the only two
# places that may take a connection from the pool, because they are what establish
# `SET LOCAL ROLE` and `app.tenant_id`. A raw `pool.get()` anywhere else runs unscoped:
# under RLS that silently returns zero rows on read, and violates NOT NULL on insert.
#
# This keeps the guarantee true as code is added. It is the counterpart to
# check-sql-boundaries.sh, which catches cross-SCHEMA SQL and would not catch this.
set -eo pipefail

cd "$(dirname "$0")/.."

echo "=========================================================="
echo "    POOL CHECKOUT AUDIT (ADR-012)                         "
echo "=========================================================="

# Two call sites legitimately need a raw connection. Each is listed with its reason; the
# list stays short, because every entry is somewhere tenant scoping cannot apply.
#
#  stheno-locking  session-level advisory locks are held on a pinned connection for the
#                  lifetime of the lock, so they cannot sit inside a committing
#                  transaction. That file namespaces its key by tenant itself (see
#                  `tenant_key`) — the one tenant boundary RLS cannot provide.
#  stheno-server/db.rs
#                  boot-time connectivity probe. Runs before any request exists, so there
#                  is no tenant to scope to, and it touches no tenant-owned table.
ALLOWED='crates/modules/stheno-locking/src/lib.rs|crates/stheno-server/src/db.rs'

# The checkout is frequently written across lines as
#     self
#         .pool
#         .get()
# so a single-line regex silently misses it. `CheckQuotaStep` was exactly that case: the
# first version of this gate reported clean while a bypass sat in the quota path. awk
# carries a one-line lookbehind so both spellings are caught.
#
# It must also catch the checkout through a LOCAL binding:
#     if let Some(pool) = &self.pool { let mut conn = pool.get().await
# Requiring a leading dot missed that shape entirely, and `WorkflowStore` sat behind the
# hole: every saga checkpoint wrote unscoped, so under RLS the insert violated NOT NULL on
# tenant_id and the durable engine persisted nothing at all. The word boundary below is
# what closes it.
FOUND=$(find crates -name '*.rs'     -not -path 'crates/core/stheno-db/src/*'     -not -path '*/tests/*'     | grep -Ev "$ALLOWED"     | while read -r f; do
        awk -v F="$f" '
            /(^|[^A-Za-z0-9_])pool[[:space:]]*\(?\)?[[:space:]]*\.[[:space:]]*get\(\)/ { print F ":" NR ": " $0; next }
            prev ~ /(^|[^A-Za-z0-9_])pool[[:space:]]*\(?\)?[[:space:]]*$/ && /^[[:space:]]*\.[[:space:]]*get\(\)/ {
                print F ":" NR ": " prev " / " $0
            }
            { prev = $0 }
        ' "$f"
      done)

if [ -n "$FOUND" ]; then
    echo "ERROR: connection checked out without a tenant scope:" >&2
    echo "$FOUND" >&2
    echo "" >&2
    echo "Use stheno_db::tenant::with_tenant_conn(&pool, |conn| ...) instead." >&2
    exit 1
fi

echo "OK: every pool checkout goes through a tenant-scoping helper."
echo "=========================================================="
exit 0
