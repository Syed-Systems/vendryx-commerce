#!/usr/bin/env bash
# P25-09 — a theme may not reach into core, or fetch for itself.
#
# The frontend equivalent of check-module-isolation.sh, and it exists for the
# same reason ADR-013 decision 9 gives: a boundary that is asked for is a
# boundary that erodes at 2am before a launch.
#
# ADR-013 decision 4: a theme supplies components against a typed contract and
# receives props. It does not fetch, and it does not route.
set -euo pipefail
cd "$(dirname "$0")/.."

PKGS="apps/storefront/packages"
FIXTURE="$PKGS/theme-fixture-violating"

# What a theme may never import: the SDK, core internals (@lib/*, @core/*), or
# core's checkout module. A theme reaches core only through the versioned
# package entry points, @stheno/storefront-core and .../server.
FORBIDDEN='@medusajs/js-sdk|@lib/|@core/|@modules/checkout'

scan() {
  # Skip commented-out imports: a line that is already disabled is not a
  # violation, and counting it inflates the debt this gate is meant to measure.
  { grep -rnE "from \"($FORBIDDEN)" "$1" --include=*.ts --include=*.tsx 2>/dev/null     | grep -vE '^[^:]+:[0-9]+:[[:space:]]*(//|\*|/\*)' ; } || true
}

# Self-test first: a check that cannot fail is not a check.
FIXTURE_HITS="$(scan "$FIXTURE")"
if [ -z "$FIXTURE_HITS" ]; then
  echo "FAIL: the violating fixture produced no findings — this check is broken." >&2
  exit 1
fi
echo "self-test: fixture flagged $(printf '%s\n' "$FIXTURE_HITS" | wc -l) violation(s), as designed."

FAIL=0
for pkg in "$PKGS"/theme-*/; do
  case "$pkg" in *theme-fixture-*) continue ;; esac
  HITS="$(scan "$pkg")"
  if [ -n "$HITS" ]; then
    COUNT="$(printf '%s\n' "$HITS" | wc -l)"
    FILES="$(printf '%s\n' "$HITS" | cut -d: -f1 | sort -u | wc -l)"
    echo "FAIL: $(basename "$pkg") reaches into core in $COUNT place(s) across $FILES file(s):" >&2
    printf '%s\n' "$HITS" | sed 's/^/  /' | head -20 >&2
    [ "$COUNT" -gt 20 ] && echo "  ... $((COUNT - 20)) more" >&2
    FAIL=1
  else
    echo "PASS: $(basename "$pkg") imports nothing from core internals or the SDK."
  fi
done
exit "$FAIL"
