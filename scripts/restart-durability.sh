#!/usr/bin/env bash
# P11-22: rows survive a real process restart.
#
# This is the single assertion Phase 1 could not make. The server boots, a
# product is created through the HTTP API, the process is **killed**, a new
# process is started against the same database, and the same product id is
# retrieved.
#
# Deliberately a real kill and a real second process. Dropping a service handle
# and building another inside one test proves the repository works; it does not
# prove the data outlived the process, because an in-memory store behind an
# `Arc` would pass that too.
set -euo pipefail

fail() { echo "FAIL: $*" >&2; exit 1; }

if [ -z "${DATABASE_URL:-}" ]; then
    fail "DATABASE_URL is unset; there is no durability to test without a database."
fi

PORT="${STHENO_TEST_PORT:-9111}"
BIND="127.0.0.1:${PORT}"
BASE="http://${BIND}"
DB_NAME="stheno_restart_$$"
SERVER_PID=""

# Swap the database name in DATABASE_URL so this drill cannot disturb anything.
ADMIN_URL="$DATABASE_URL"
BASE_URL="${ADMIN_URL%%\?*}"
QUERY=""
[ "$ADMIN_URL" != "$BASE_URL" ] && QUERY="?${ADMIN_URL#*\?}"
TEST_URL="${BASE_URL%/*}/${DB_NAME}${QUERY}"

cleanup() {
    [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true
    psql_admin "DROP DATABASE IF EXISTS \"$DB_NAME\" WITH (FORCE)" >/dev/null 2>&1 || true
}
trap cleanup EXIT

psql_admin() {
    cargo run --quiet -p stheno-cli -- exec-sql --url "$ADMIN_URL" --sql "$1" 2>/dev/null \
        || docker exec stheno-pg psql -U stheno -d stheno -c "$1" >/dev/null
}

wait_for_health() {
    for _ in $(seq 1 60); do
        if curl -fsS "${BASE}/health" >/dev/null 2>&1; then
            return 0
        fi
        sleep 1
    done
    return 1
}

start_server() {
    DATABASE_URL="$TEST_URL" ./target-bin/stheno-server --bind "$BIND" >"/tmp/stheno-restart-$$.log" 2>&1 &
    SERVER_PID=$!
    wait_for_health || {
        echo "--- server log ---" >&2
        cat "/tmp/stheno-restart-$$.log" >&2
        fail "server did not become healthy on ${BASE}"
    }
}

echo "==> Restart durability drill"

echo "[1/7] Building the server binary..."
cargo build --quiet -p stheno-server
mkdir -p target-bin
BIN=$(cargo build -p stheno-server --message-format=json 2>/dev/null \
    | grep -o '"executable":"[^"]*stheno-server[^"]*"' \
    | head -1 | sed 's/.*":"//;s/"$//' | sed 's|\\\\|/|g')
[ -n "$BIN" ] && [ -f "$BIN" ] || fail "could not locate the built stheno-server binary"
cp "$BIN" target-bin/stheno-server

echo "[2/7] Provisioning an isolated database ($DB_NAME)..."
docker exec stheno-pg psql -U stheno -d stheno -c "DROP DATABASE IF EXISTS \"$DB_NAME\" WITH (FORCE)" >/dev/null
docker exec stheno-pg psql -U stheno -d stheno -c "CREATE DATABASE \"$DB_NAME\"" >/dev/null

echo "[3/7] Migrating and booting the first process..."
DATABASE_URL="$TEST_URL" ./target-bin/stheno-server --migrate --check-migrations \
    || fail "migrations did not apply cleanly"
start_server
FIRST_PID="$SERVER_PID"
echo "    first process pid=$FIRST_PID"

echo "[4/7] Creating a product over HTTP..."
CREATED=$(curl -fsS -X POST "${BASE}/admin/products" \
    -H 'Content-Type: application/json' \
    -d '{"title":"Restart Durability Probe","handle":"restart-durability-probe"}') \
    || fail "could not create a product"

PRODUCT_ID=$(echo "$CREATED" | grep -o '"id":"[^"]*"' | head -1 | sed 's/.*":"//;s/"$//')
[ -n "$PRODUCT_ID" ] || fail "the create response carried no id: $CREATED"
echo "    created $PRODUCT_ID"

case "$PRODUCT_ID" in
    prod_*) ;;
    *) fail "the product id lost its Medusa prefix: '$PRODUCT_ID' (ADR-010)" ;;
esac

echo "[5/7] Killing the process..."
kill -9 "$FIRST_PID" 2>/dev/null || true
wait "$FIRST_PID" 2>/dev/null || true
SERVER_PID=""
# The port must actually be released, or step 6 would silently talk to a
# process that never died and prove nothing.
for _ in $(seq 1 30); do
    curl -fsS "${BASE}/health" >/dev/null 2>&1 || break
    sleep 1
done
curl -fsS "${BASE}/health" >/dev/null 2>&1 && fail "the first process is still serving; it was not killed"
echo "    process $FIRST_PID is gone"

echo "[6/7] Starting a second process against the same database..."
start_server
echo "    second process pid=$SERVER_PID"
[ "$SERVER_PID" != "$FIRST_PID" ] || fail "the pid did not change; no restart happened"

echo "[7/7] Retrieving the same product id..."
FETCHED=$(curl -fsS "${BASE}/admin/products/${PRODUCT_ID}") \
    || fail "the product did not survive the restart: ${PRODUCT_ID}"

echo "$FETCHED" | grep -q "$PRODUCT_ID" \
    || fail "the retrieved product is not the one created: $FETCHED"
echo "$FETCHED" | grep -q "Restart Durability Probe" \
    || fail "the product came back without its title: $FETCHED"

echo "    retrieved $PRODUCT_ID after restart"
echo "Restart durability PASSED: a product created before a kill -9 was read back by a new process."
exit 0
