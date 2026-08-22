#!/usr/bin/env bash
# P25-11 — every frontend pins a storefront-core major.
#
# Enforces the mechanical half of docs/FRONTEND_VERSIONING.md: a frontend must declare
# @stheno/storefront-core as a peerDependency, and the major it pins must be the
# core actually present in the workspace. It cannot enforce that a core minor was
# genuinely additive — ADR-013 says so plainly, and that half stays human.
set -euo pipefail
cd "$(dirname "$0")/.."

PKGS="apps/storefront/packages"
CORE_VERSION="$(node -p "require('./$PKGS/storefront-core/package.json').version")"
CORE_MAJOR="${CORE_VERSION%%.*}"
FAIL=0
FOUND=0

for pkg in "$PKGS"/frontend-*/package.json; do
  [ -f "$pkg" ] || continue
  FOUND=$((FOUND + 1))
  name="$(node -p "require('./$pkg').name")"
  pin="$(node -p "(require('./$pkg').peerDependencies||{})['@stheno/storefront-core']||''")"

  if [ -z "$pin" ]; then
    echo "FAIL: $name does not pin @stheno/storefront-core in peerDependencies." >&2
    FAIL=1
    continue
  fi

  pin_major="$(printf '%s' "$pin" | sed 's/^[^0-9]*//' | cut -d. -f1)"
  if [ "$pin_major" != "$CORE_MAJOR" ]; then
    echo "FAIL: $name pins core $pin but the workspace core is $CORE_VERSION." >&2
    FAIL=1
    continue
  fi
  echo "  $name pins $pin (core $CORE_VERSION)"
done

if [ "$FOUND" -eq 0 ]; then
  echo "FAIL: no frontend packages found under $PKGS." >&2
  exit 1
fi

if [ "$FAIL" -eq 0 ]; then
  echo "PASS: $FOUND frontend(s) pin core $CORE_VERSION."
fi
exit "$FAIL"
