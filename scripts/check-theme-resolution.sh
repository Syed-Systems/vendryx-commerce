#!/usr/bin/env bash
# P25-07 — core binds a theme in exactly one place.
#
# The binding is the @theme/* path mapping in apps/storefront/tsconfig.json.
# Core imports theme components as @theme/<path>; nothing in core names the
# theme package itself. Swapping themes is that one line.
#
# Why an alias and not a re-export barrel: a barrel was built first and failed.
# Any client component importing it pulls every other slot into the client
# graph, including server-only data modules behind the footer — Next fails the
# build outright. A runtime component map is worse and is what ADR-013 decision
# 5 already rejected. Per-module imports through one alias keep both the RSC
# boundary and tree-shaking intact.
set -euo pipefail
cd "$(dirname "$0")/.."

APP="apps/storefront"
CORE="$APP/packages/storefront-core/src"
TSCONFIG="$APP/tsconfig.json"
FAIL=0

BINDINGS="$(grep -c '"@theme/\*"' "$TSCONFIG" || true)"
if [ "$BINDINGS" -ne 1 ]; then
  echo "FAIL: expected exactly one @theme/* binding in $TSCONFIG, found $BINDINGS." >&2
  FAIL=1
fi

# Naming the package directly bypasses the binding.
DIRECT="$(grep -rln '@stheno/theme-default' "$CORE" || true)"
if [ -n "$DIRECT" ]; then
  echo "FAIL: core modules name the theme package instead of using @theme/*:" >&2
  echo "$DIRECT" | sed 's/^/  /' >&2
  FAIL=1
fi

# The contract must be enforced, or a theme could omit a slot and still build.
[ -f "$CORE/theme/contract.ts" ] || { echo "FAIL: no theme contract." >&2; FAIL=1; }
[ -f "$CORE/theme/conformance.ts" ] || { echo "FAIL: contract is not enforced." >&2; FAIL=1; }

SLOTS="$(grep -c '^  "' "$CORE/theme/contract.ts" || true)"
if [ "$SLOTS" -lt 1 ]; then
  echo "FAIL: contract enumerates no slots." >&2
  FAIL=1
fi

if [ "$FAIL" -eq 0 ]; then
  echo "PASS: one @theme/* binding, $SLOTS contract slots, conformance enforced at typecheck."
fi
exit "$FAIL"
