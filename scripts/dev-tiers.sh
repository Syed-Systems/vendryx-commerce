#!/usr/bin/env bash
# Stands up one deployment per plan tier for end-to-end testing.
#
# ADR-009 and ENTITLEMENTS.md 2: one binary and one database per store. A tier is
# therefore a *deployment*, not a login — three tiers means three processes and three
# databases, and the isolation is physical. Anything that pretends otherwise (three
# accounts inside one database) would be testing a product we did not build.
#
#   ./scripts/dev-tiers.sh up      migrate, seed, create owners, start all three
#   ./scripts/dev-tiers.sh down    stop them
#   ./scripts/dev-tiers.sh status  show ports, tiers, and owner credentials
#
# Requires: a running Postgres and `cargo build -p stheno-server`.
set -euo pipefail

cd "$(dirname "$0")/.."

PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-stheno}"
PGPASSWORD="${PGPASSWORD:-stheno}"
export PGPASSWORD

BIN="${STHENO_BIN:-$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/debug/stheno-server}"
[ -x "$BIN" ] || BIN="$BIN.exe"

RUNTIME_DIR=".dev-tiers"
PASSWORD="${STHENO_DEV_PASSWORD:-Sup3rSecret!}"

# tier:port:database:owner-email
TIERS=(
  "trial:9001:stheno_trial:owner-trial@stheno.test"
  "free:9002:stheno_free:owner-free@stheno.test"
  "pro:9003:stheno_pro:owner-pro@stheno.test"
)

# Local psql if it exists, otherwise the one inside the compose container — the repo's
# docker-compose.yml is how most people here get Postgres, and it already ships a client.
if command -v psql >/dev/null 2>&1; then
  psql_admin() { psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d postgres -v ON_ERROR_STOP=1 "$@"; }
else
  psql_admin() {
    docker compose exec -T postgres psql -U "$PGUSER" -d postgres -v ON_ERROR_STOP=1 "$@"
  }
fi

require_binary() {
  if [ ! -x "$BIN" ]; then
    echo "stheno-server not built at $BIN — run: cargo build -p stheno-server" >&2
    exit 1
  fi
}

wait_for_health() {
  local port=$1 tier=$2
  for _ in $(seq 1 60); do
    if curl -fsS -m 2 "http://127.0.0.1:${port}/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  echo "[$tier] did not become healthy on :$port — see $RUNTIME_DIR/$tier.log" >&2
  return 1
}

up() {
  require_binary
  mkdir -p "$RUNTIME_DIR"

  for entry in "${TIERS[@]}"; do
    IFS=: read -r tier port db email <<<"$entry"
    url="postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${db}"

    echo "==> $tier  :$port  db=$db"

    psql_admin -tAc "SELECT 1 FROM pg_database WHERE datname='${db}'" | grep -q 1 \
      || psql_admin -c "CREATE DATABASE \"${db}\""

    # `--migrate` applies pending migrations and then serves — it is a startup flag,
    # not a subcommand that exits. So it goes on the long-running process rather than
    # a separate step that would never return.
    DATABASE_URL="$url" STHENO_TIER="$tier" \
    ADMIN_CORS="http://localhost:5173,http://127.0.0.1:5173" \
    STORE_CORS="http://localhost:8000,http://127.0.0.1:8000" \
    AUTH_CORS="http://localhost:5173,http://127.0.0.1:5173,http://localhost:8000,http://127.0.0.1:8000" \
      nohup "$BIN" --migrate --bind "127.0.0.1:${port}" >"$RUNTIME_DIR/$tier.log" 2>&1 &
    echo $! >"$RUNTIME_DIR/$tier.pid"

    wait_for_health "$port" "$tier"

    # Seeding is its own short-lived process against the same database.
    DATABASE_URL="$url" "$BIN" --seed-demo >/dev/null 2>&1 || true

    # The owner account. Registration is idempotent in effect: a second run gets a
    # conflict, which is fine — the credentials below still work.
    curl -fsS -m 10 -X POST "http://127.0.0.1:${port}/auth/user/emailpass/register" \
      -H 'Content-Type: application/json' \
      -d "{\"email\":\"${email}\",\"password\":\"${PASSWORD}\"}" >/dev/null 2>&1 || true

    echo "    owner: ${email} / ${PASSWORD}"
  done

  echo
  status
}

down() {
  for entry in "${TIERS[@]}"; do
    IFS=: read -r tier _ _ _ <<<"$entry"
    pidfile="$RUNTIME_DIR/$tier.pid"
    [ -f "$pidfile" ] || continue
    kill "$(cat "$pidfile")" 2>/dev/null || true
    rm -f "$pidfile"
    echo "stopped $tier"
  done
}

status() {
  printf '%-7s %-6s %-16s %-28s %s\n' TIER PORT DATABASE OWNER STATE
  for entry in "${TIERS[@]}"; do
    IFS=: read -r tier port db email <<<"$entry"
    if curl -fsS -m 2 "http://127.0.0.1:${port}/health" >/dev/null 2>&1; then
      state="up"
    else
      state="down"
    fi
    printf '%-7s %-6s %-16s %-28s %s\n' "$tier" "$port" "$db" "$email" "$state"
  done
  echo
  echo "password for every owner: $PASSWORD"
}

case "${1:-up}" in
  up) up ;;
  down) down ;;
  status) status ;;
  *) echo "usage: $0 {up|down|status}" >&2; exit 2 ;;
esac
