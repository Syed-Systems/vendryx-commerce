#!/usr/bin/env bash
# P11-02: the dashboard upstream pin is real.
#
# This check exists because the pin was fabricated once already: the previous
# UPSTREAM.md named a repository that does not exist and a commit that was never
# in it, and nothing caught that for a whole phase. So the pin is not read and
# trusted here -- it is re-resolved against the actual remote.
set -euo pipefail

UPSTREAM="apps/admin/UPSTREAM.md"

fail() { echo "FAIL: $*" >&2; exit 1; }

echo "==> Checking dashboard upstream pin..."

[ -f "$UPSTREAM" ] || fail "$UPSTREAM is missing."

# Pull the three fields out of the pin table.
REPO=$(grep -E '^\| Repository \|' "$UPSTREAM" | sed -E 's/.*`([^`]+)`.*/\1/')
TAG=$(grep -E '^\| Tag \|' "$UPSTREAM" | sed -E 's/.*`([^`]+)`.*/\1/')
COMMIT=$(grep -E '^\| Commit \|' "$UPSTREAM" | sed -E 's/.*`([^`]+)`.*/\1/')

[ -n "$REPO" ]   || fail "$UPSTREAM does not record a repository."
[ -n "$TAG" ]    || fail "$UPSTREAM does not record a tag."
[ -n "$COMMIT" ] || fail "$UPSTREAM does not record a commit."

# A full SHA-1, not an abbreviated one. The fabricated pin was 7 characters,
# which is short enough to look plausible and impossible to verify by eye.
echo "$COMMIT" | grep -qE '^[0-9a-f]{40}$' \
    || fail "commit '$COMMIT' is not a full 40-character SHA."

if ! command -v git >/dev/null 2>&1; then
    fail "git is not installed; the pin cannot be verified."
fi

echo "    repository: $REPO"
echo "    tag:        $TAG"
echo "    commit:     $COMMIT"

# Resolve the tag on the real remote. This is the step that fails loudly for a
# repository that does not exist.
if ! REMOTE_OUT=$(git ls-remote --tags "$REPO" "refs/tags/$TAG" 2>&1); then
    fail "cannot reach $REPO -- the repository may not exist:
$REMOTE_OUT"
fi

RESOLVED=$(echo "$REMOTE_OUT" | grep -v '\^{}' | awk 'NR==1 {print $1}')
[ -n "$RESOLVED" ] || fail "tag '$TAG' does not exist in $REPO."

[ "$RESOLVED" = "$COMMIT" ] \
    || fail "pin drift: $TAG resolves to $RESOLVED, but $UPSTREAM records $COMMIT."

# The dashboard is a subdirectory of the monorepo, not a repository of its own.
# Recording the wrong shape here is how the original fabrication started.
grep -q 'packages/admin/dashboard' "$UPSTREAM" \
    || fail "$UPSTREAM does not record the dashboard's subdirectory in the monorepo."

echo "Upstream pin check PASSED: $TAG resolves to $COMMIT in $REPO."
exit 0
