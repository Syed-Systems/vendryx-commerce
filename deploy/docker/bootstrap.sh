#!/bin/sh
# Docker one-shot bootstrap: waits for the engine's migrations, then creates the
# trial/free/pro tenants, seeds demo catalogs and registers owners.
#
# This mirrors scripts/dev-tenants.sh, which remains the local-dev path
# (cargo debug binary + npm run dev hot-reload). This script is the container
# path: release binary, Docker DNS names, no Git Bash / hosts-file edits.
set -eu

BIN="/app/target/release/stheno-server"
ENGINE="${STHENO_ENGINE_URL:-http://engine:9000}"
PASSWORD="${STHENO_DEV_PASSWORD:-Sup3rSecret!}"

echo "==> waiting for engine at ${ENGINE} ..."
for _ in $(seq 1 180); do
  if curl -fsS -m 2 "${ENGINE}/health" >/dev/null 2>&1; then break; fi
  sleep 1
done
curl -fsS -m 5 "${ENGINE}/health" >/dev/null
echo "    engine is up"

echo "==> waiting for migrations ..."
for _ in $(seq 1 60); do
  if "$BIN" --check-migrations >/dev/null 2>&1; then break; fi
  sleep 2
done
"$BIN" --check-migrations >/dev/null
echo "    schema is current"

create_tenant() {
  id="$1"; name="$2"; tier="$3"; domain="$4"
  echo "==> ${id} (${tier}) at http://${domain}"
  if ! out=$("$BIN" --create-tenant "${id}:${name}:${tier}:${domain}" 2>&1); then
    case "$out" in
      *already*|*exists*|*conflict*|*duplicate*) ;;
      *) echo "    !! could not create ${id}: ${out}" ;;
    esac
  fi
  if ! out=$(STHENO_TENANT="$id" "$BIN" --seed-demo 2>&1); then
    echo "    !! could not seed ${id}: ${out}"
  fi
  # Owner registration is Host-scoped, so each tenant gets its own.
  curl -fsS -m 10 -X POST "${ENGINE}/auth/user/emailpass/register" \
    -H 'Content-Type: application/json' -H "Host: ${domain}" \
    -d "{\"email\":\"owner@${domain}\",\"password\":\"${PASSWORD}\"}" >/dev/null 2>&1 || true
  echo "    owner: owner@${domain}"
}

create_tenant "ten_trial" "Trial Merchant" "trial" "trial.localhost"
create_tenant "ten_free" "Free Merchant" "free" "free.localhost"
create_tenant "ten_pro" "Pro Merchant" "pro" "pro.localhost"

# Secondary domains for callers that cannot resolve *.localhost. Node SSR and the
# middleware fetch ${STHENO_BACKEND_URL}/store/regions WITHOUT forwarding the
# incoming Host header, so the engine sees Host: engine:9000 (Docker DNS) and 404s
# "Not found" unless `engine` itself is a tenant. host.docker.internal is what the
# storefront *builder* reaches during `next build` prerender, so it must resolve a
# tenant too or generateStaticParams 404s. Same rows dev-tenants.sh writes;
# platform.* sits outside the RLS loop.
psql "$DATABASE_URL" -qc "INSERT INTO platform.tenant_domain (domain, tenant_id, is_primary) VALUES ('127.0.0.1', 'ten_pro', false), ('host.docker.internal', 'ten_pro', false), ('engine', 'ten_pro', false) ON CONFLICT (domain) DO NOTHING" >/dev/null 2>&1 || true

echo "bootstrap complete — password for every owner: ${PASSWORD}"
