#!/usr/bin/env bash
# P25-10 — a frontend presents; it does not fetch, mutate the world, or own flow.
#
# ADR-013 decision 4: a frontend supplies components against a typed contract and
# receives props. It does not fetch, and it does not route.
#
# check-storefront-boundaries.sh (P25-09) governs *where* a frontend may import
# from. This governs *what* it may do. A frontend can satisfy the boundary rule by
# importing a read path from core's public server entry and still be fetching,
# which is exactly what this catches.
set -euo pipefail
cd "$(dirname "$0")/.."

PKGS="apps/storefront/packages"
FIXTURE="$PKGS/frontend-fixture-violating"
FAIL=0

probe() {
  { grep -rnE "$2" "$1" --include=*.ts --include=*.tsx 2>/dev/null \
    | grep -vE '^[^:]+:[0-9]+:[[:space:]]*(//|\*|/\*)' ; } || true
}

# A frontend reads by importing core's server entry, declaring its own server
# action, or reaching for request state directly.
READS='from "@stheno/storefront-core/server"'
SERVER_STATE='"use server"|next/headers|"server-only"'

check_theme() {
  local pkg="$1" name found=0
  name="$(basename "$pkg")"

  local hits
  hits="$(probe "$pkg" "$READS")"
  if [ -n "$hits" ]; then
    echo "FAIL: $name fetches. Lift these reads into core and pass the result as props:" >&2
    printf '%s\n' "$hits" | sed 's/^/  /' >&2
    found=1
  fi

  hits="$(probe "$pkg" "$SERVER_STATE")"
  if [ -n "$hits" ]; then
    echo "FAIL: $name declares server-side behaviour:" >&2
    printf '%s\n' "$hits" | sed 's/^/  /' >&2
    found=1
  fi

  if [ "$found" -eq 0 ]; then
    echo "PASS: $name presents only - no reads, no server actions, no request state."
  fi
  return "$found"
}

# Self-test: a check that cannot fail is not a check.
if check_theme "$FIXTURE" >/dev/null 2>&1; then
  echo "FAIL: the violating fixture passed the purity check - this check is broken." >&2
  exit 1
fi
echo "self-test: fixture rejected, as designed."

for pkg in "$PKGS"/frontend-*/; do
  case "$pkg" in *frontend-fixture-*) continue ;; esac
  check_theme "$pkg" || FAIL=1
done
exit "$FAIL"
