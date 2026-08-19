#!/usr/bin/env bash
# Stands up ONE server serving three tenants on three plan tiers (ADR-012).
#
# This replaces scripts/dev-tiers.sh, which ran three processes against three databases
# because ADR-009 made a tier a property of a deployment. A tier is now a property of a
# subscription row, so one process serves all three and isolation is enforced by RLS.
#
#   ./scripts/dev-tenants.sh up      migrate, create tenants, seed, start the server
#   ./scripts/dev-tenants.sh down    stop it
#   ./scripts/dev-tenants.sh status  show tenants, tiers and URLs
#
# Tenants are told apart by Host, so each gets a `.localhost` name.
#
# `*.localhost` resolves to loopback in *browsers*, which handle it internally — that is
# why the admin's Playwright suite reaches these hosts. It does NOT resolve in Node,
# which asks the OS resolver, and Windows special-cases only bare `localhost`. The
# storefront's server-side fetches run in Node, so `pro.localhost` fails there with
# ENOTFOUND. Rather than require a hosts-file edit (and elevation), the pro tenant also
# claims the literal loopback address as a second domain — a tenant may hold many
# domains, so this needs no new mechanism. Server-side callers use IP_DOMAIN below.
set -euo pipefail

cd "$(dirname "$0")/.."

PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-stheno}"
PGPASSWORD="${PGPASSWORD:-stheno}"
export PGPASSWORD

DB="${STHENO_DEV_DB:-stheno_dev}"
PORT="${STHENO_DEV_PORT:-9000}"
DATABASE_URL="postgres://${PGUSER}:${PGPASSWORD}@${PGHOST}:${PGPORT}/${DB}"
export DATABASE_URL

BIN="${STHENO_BIN:-$(cargo metadata --format-version 1 --no-deps 2>/dev/null \
  | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/debug/stheno-server}"
[ -x "$BIN" ] || BIN="$BIN.exe"

RUNTIME_DIR=".dev-tenants"
PASSWORD="${STHENO_DEV_PASSWORD:-Sup3rSecret!}"

# id:name:tier:domain
TENANTS=(
  "ten_trial:Trial Merchant:trial:trial.localhost"
  "ten_free:Free Merchant:free:free.localhost"
  "ten_pro:Pro Merchant:pro:pro.localhost"
)

# The tenant reached over the raw loopback address, for callers that cannot resolve
# `*.localhost` (Node). Pro, because it is the tier with every feature mounted.
IP_DOMAIN_TENANT="ten_pro"
IP_DOMAIN="127.0.0.1"

psql_dev() {
  if command -v psql >/dev/null 2>&1; then
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$DB" -v ON_ERROR_STOP=1 "$@"
  else
    docker compose exec -T postgres psql -U "$PGUSER" -d "$DB" -v ON_ERROR_STOP=1 "$@"
  fi
}

psql_admin() {
  if command -v psql >/dev/null 2>&1; then
    psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d postgres -v ON_ERROR_STOP=1 "$@"
  else
    docker compose exec -T postgres psql -U "$PGUSER" -d postgres -v ON_ERROR_STOP=1 "$@"
  fi
}

up() {
  if [ ! -x "$BIN" ]; then
    echo "stheno-server not built at $BIN — run: cargo build -p stheno-server" >&2
    exit 1
  fi
  mkdir -p "$RUNTIME_DIR"

  psql_admin -tAc "SELECT 1 FROM pg_database WHERE datname='${DB}'" | grep -q 1 \
    || psql_admin -c "CREATE DATABASE \"${DB}\""

  # `--migrate` applies migrations and then serves, so it goes on the long-running
  # process rather than a separate step that would never return.
  STHENO_PLATFORM_TOKEN="${STHENO_PLATFORM_TOKEN:-dev-platform-token}" \
  ADMIN_CORS="http://localhost:5173,http://127.0.0.1:5173,http://localhost:5174,http://127.0.0.1:5174" \
  STORE_CORS="http://localhost:8000,http://127.0.0.1:8000" \
  AUTH_CORS="http://localhost:5173,http://127.0.0.1:5173,http://localhost:8000,http://127.0.0.1:8000" \
    nohup "$BIN" --migrate --bind "127.0.0.1:${PORT}" >"$RUNTIME_DIR/server.log" 2>&1 &
  echo $! >"$RUNTIME_DIR/server.pid"

  for _ in $(seq 1 90); do
    curl -fsS -m 2 "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1 && break
    sleep 1
  done

  for entry in "${TENANTS[@]}"; do
    IFS=: read -r id name tier domain <<<"$entry"
    echo "==> $id ($tier) at http://${domain}:${PORT}"

    "$BIN" --create-tenant "${id}:${name}:${tier}:${domain}" >/dev/null 2>&1 || true
    STHENO_TENANT="$id" "$BIN" --seed-demo >/dev/null 2>&1 || true

    # The owner account. Registration is scoped by Host, so each tenant gets its own —
    # and the same email may legitimately be reused across tenants now that the unique
    # index carries tenant_id.
    curl -fsS -m 10 -X POST "http://127.0.0.1:${PORT}/auth/user/emailpass/register" \
      -H 'Content-Type: application/json' -H "Host: ${domain}" \
      -d "{\"email\":\"owner@${domain}\",\"password\":\"${PASSWORD}\"}" >/dev/null 2>&1 || true

    echo "    owner: owner@${domain} / ${PASSWORD}"
  done

  # Secondary, so `is_primary` stays with the real hostname and nothing starts printing
  # an IP address as the merchant's address. platform.* is outside the RLS loop — it is
  # the registry RLS is resolved *from* — so this is a plain insert.
  psql_dev -qc "INSERT INTO platform.tenant_domain (domain, tenant_id, is_primary)
                VALUES ('${IP_DOMAIN}', '${IP_DOMAIN_TENANT}', false)
                ON CONFLICT (domain) DO NOTHING" >/dev/null 2>&1 || true

  echo
  status
}

down() {
  pidfile="$RUNTIME_DIR/server.pid"
  [ -f "$pidfile" ] || { echo "not running"; return 0; }
  kill "$(cat "$pidfile")" 2>/dev/null || true
  rm -f "$pidfile"
  echo "stopped"
}

status() {
  if curl -fsS -m 2 "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
    echo "server: up on :${PORT}  db=${DB}"
  else
    echo "server: down"
    return 0
  fi
  printf '%-12s %-8s %-22s %s\n' TENANT TIER HOST OWNER
  for entry in "${TENANTS[@]}"; do
    IFS=: read -r id name tier domain <<<"$entry"
    printf '%-12s %-8s %-22s %s\n' "$id" "$tier" "${domain}:${PORT}" "owner@${domain}"
  done
  echo
  echo "password for every owner: $PASSWORD"
  echo "${IP_DOMAIN}:${PORT} also resolves to ${IP_DOMAIN_TENANT} (for Node callers)"
}

case "${1:-up}" in
  up) up ;;
  down) down ;;
  status) status ;;
  *) echo "usage: $0 {up|down|status}" >&2; exit 2 ;;
esac
