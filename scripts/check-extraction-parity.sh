#!/usr/bin/env bash
# P25-13 [BP] — proves the core/theme extraction changed no rendered output.
#
#   ./scripts/check-extraction-parity.sh --capture   record the pre-extraction baseline
#   ./scripts/check-extraction-parity.sh             compare current render to it
#
# The baseline is captured BEFORE any code moves and committed. Every later P25
# task re-runs the comparison. A theme or core edit that changes what a shopper
# sees fails here, which is the whole point of the phase's "zero behaviour
# change" exit gate.
set -euo pipefail

cd "$(dirname "$0")/.."
APP="apps/storefront"
PORT="${PARITY_PORT:-8100}"
ROUTES="$APP/tests/parity/routes.txt"
BASELINE="$APP/tests/parity/route-snapshots.sha256"
NORMALIZE="$APP/tests/parity/normalize.mjs"
MODE="${1:---check}"

# Returns the PIDs listening on $1, on Git Bash/Windows or a POSIX host.
listeners_on() {
  if command -v netstat >/dev/null 2>&1 && netstat -ano >/dev/null 2>&1; then
    { netstat -ano | grep ":$1 " | grep -i LISTENING | awk '{print $5}' | sort -u; } || true
  elif command -v lsof >/dev/null 2>&1; then
    lsof -ti ":$1" 2>/dev/null || true
  fi
}

kill_pid_tree() {
  local pid="$1"
  [ -z "$pid" ] && return 0
  if command -v taskkill >/dev/null 2>&1; then
    taskkill //PID "$pid" //T //F >/dev/null 2>&1 || true
  else
    kill -9 "$pid" >/dev/null 2>&1 || true
  fi
}

# The catalog is a fixture, and it moves: e2e runs create products, so two
# builds minutes apart can render different product lists through identical
# code. Fingerprint it, so a changed fixture reports as a changed fixture
# instead of masquerading as a rendering regression.
BACKEND="$(grep -E '^STHENO_BACKEND_URL=' "$APP/.env.local" | cut -d= -f2- | tr -d '
')"
PUBKEY="$(grep -E '^NEXT_PUBLIC_STHENO_PUBLISHABLE_KEY=' "$APP/.env.local" | cut -d= -f2- | tr -d '
')"
catalog_fingerprint() {
  curl -sf -H "x-publishable-api-key: $PUBKEY" "$BACKEND/store/products?limit=200&fields=handle" 2>/dev/null     | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{try{const p=JSON.parse(s).products||[];console.log(p.map(x=>x.handle).sort().join(","))}catch{console.log("UNAVAILABLE")}})'     | sha256sum | cut -d" " -f1
}
CATALOG="$(catalog_fingerprint)"

if [ "$MODE" != "--capture" ] && [ ! -f "$BASELINE" ]; then
  echo "FAIL: no baseline at $BASELINE. Run --capture before extracting." >&2
  exit 1
fi

# .next must go. Next caches prerendered SSG pages and fetch results between
# builds, so an incremental build can serve HTML generated from source that no
# longer exists — a parity check reading that cache passes on stale output and
# proves nothing. Verified: a reverted edit still rendered its perturbed markup
# until the cache was cleared.
# A server already holding the port would answer with its own build and
# silently invalidate every comparison below. Killing the launcher subshell does
# not kill `next start`, so orphans outlive a run and are the reason this guard
# exists: verified against a deliberate perturbation that the check reported as
# PASS while a survivor served the previous build.
EXISTING="$(listeners_on "$PORT")"
if [ -n "$EXISTING" ]; then
  echo "FAIL: port $PORT already in use (pid $EXISTING). Free it or set PARITY_PORT." >&2
  exit 1
fi

echo "Building storefront (clean)..."
rm -rf "$APP/.next"
npm --prefix "$APP" run build >/dev/null 2>&1 || { echo "FAIL: build failed" >&2; exit 1; }
BUILD_ID="$(cat "$APP/.next/BUILD_ID")"

echo "Starting storefront on :$PORT (build $BUILD_ID)..."
(cd "$APP" && npx next start -p "$PORT" >/dev/null 2>&1) &
LAUNCHER_PID=$!
cleanup() {
  for pid in $(listeners_on "$PORT"); do kill_pid_tree "$pid"; done
  kill "$LAUNCHER_PID" 2>/dev/null || true
  rm -f "${JAR:-}" 2>/dev/null || true
}
trap cleanup EXIT

# The region middleware answers the first request with a 307 to the same URL,
# carrying the _stheno_cache_id cookie. Without a jar every request loops.
JAR="$(mktemp)"
for _ in $(seq 1 60); do
  if curl -sfL --max-redirs 5 -c "$JAR" -b "$JAR" -o /dev/null "http://127.0.0.1:$PORT/us"; then break; fi
  sleep 1
done
FIRST="$(curl -sfL --max-redirs 5 -c "$JAR" -b "$JAR" "http://127.0.0.1:$PORT/us")"   || { echo "FAIL: server never became ready" >&2; exit 1; }

# The served page must come from the build just made, not from a survivor.
if ! printf '%s' "$FIRST" | grep -q "$BUILD_ID"; then
  echo "FAIL: server on :$PORT is not serving build $BUILD_ID - stale process." >&2
  exit 1
fi

ACTUAL="$(mktemp)"
while read -r route; do
  case "$route" in ''|'#'*) continue ;; esac
  body="$(curl -sL --max-redirs 5 -c "$JAR" -b "$JAR" "http://127.0.0.1:$PORT$route")"
  norm="$(printf '%s' "$body" | node "$NORMALIZE")"
  # PARITY_DUMP=<dir> keeps the normalised bodies so a failure can be diffed
  # instead of guessed at.
  if [ -n "${PARITY_DUMP:-}" ]; then
    mkdir -p "$PARITY_DUMP"
    printf '%s' "$norm" > "$PARITY_DUMP/$(printf '%s' "$route" | tr '/' '_').html"
  fi
  hash="$(printf '%s' "$norm" | sha256sum | cut -d' ' -f1)"
  printf '%s  %s\n' "$hash" "$route" >> "$ACTUAL"
done < "$ROUTES"

if [ "$MODE" = "--capture" ]; then
  { echo "# catalog $CATALOG"; cat "$ACTUAL"; } > "$BASELINE"
  rm -f "$ACTUAL"
  echo "PASS: baseline captured — $(grep -vc '^# catalog ' "$BASELINE") routes, catalog $CATALOG"
  exit 0
fi

BASE_CATALOG="$(grep '^# catalog ' "$BASELINE" | awk '{print $3}')"
if [ "$BASE_CATALOG" != "$CATALOG" ]; then
  echo "FAIL: the catalog fixture changed since the baseline was captured." >&2
  echo "      baseline $BASE_CATALOG != current $CATALOG" >&2
  echo "      This is not a rendering regression. Quiesce the backend and re-capture." >&2
  rm -f "$ACTUAL"
  exit 1
fi

if diff -u <(grep -v '^# catalog ' "$BASELINE") "$ACTUAL"; then
  echo "PASS: all $(grep -vc '^# catalog ' "$BASELINE") routes render byte-identically to the pre-extraction baseline."
  rm -f "$ACTUAL"
  exit 0
fi

echo "FAIL: rendered output changed. The extraction was not behaviour-neutral." >&2
rm -f "$ACTUAL"
exit 1
