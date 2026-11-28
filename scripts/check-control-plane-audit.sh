#!/usr/bin/env bash
# P29-08: every control-plane write is attributed to the operator who made it.
#
# Under ADR-012 `/platform/*` is the one surface that reads and writes across every
# merchant, and every write it makes is billing-affecting. P29-05 audited the five write
# handlers that existed at the time. This script is what stops handler number nineteen
# from shipping unaudited because its author did not read CONTROL_PLANE.md §5.
#
# The rule: a handler registered on POST, PUT, PATCH or DELETE must call `record(`.
# Read handlers are exempt — until impersonation lands (P34-03), at which point sensitive
# reads join the list and this script grows a second rule.
set -euo pipefail

CRATE="crates/api/stheno-api-platform"
ROUTER="${CRATE}/src/lib.rs"

# Extracts the handlers registered on a mutating method from a router definition.
#
# Matches `post(create_tenant)`, and the chained `get(list).post(create)` form, which is
# why this is a grep over the whole file rather than a line-oriented parse.
write_handlers() {
    grep -oE '\b(post|put|patch|delete)\([a-z_]+\)' "$1" \
        | sed -E 's/^[a-z]+\(([a-z_]+)\)$/\1/' \
        | sort -u
}

# Prints the body of `pub async fn <name>` from a source tree, or nothing if absent.
#
# Terminates on a closing brace in column zero, which is rustfmt's guarantee for a
# top-level item and the reason this does not need to balance braces.
handler_body() {
    local dir="$1" name="$2" file
    for file in $(find "$dir" -name '*.rs' 2>/dev/null); do
        if grep -q "pub async fn ${name}(" "$file"; then
            awk -v marker="pub async fn ${name}(" '
                index($0, marker) { inside = 1 }
                inside            { print }
                inside && /^}/    { exit }
            ' "$file"
            return 0
        fi
    done
    return 1
}

# Returns 0 when every mutating handler in the tree records an audit row.
audit_check() {
    local dir="$1" router="$2" failed=0 checked=0 handler body

    for handler in $(write_handlers "$router"); do
        if ! body=$(handler_body "$dir" "$handler"); then
            echo "FAIL: ${handler} is routed as a write but has no 'pub async fn ${handler}' in ${dir}" >&2
            failed=1
            continue
        fi

        checked=$((checked + 1))

        # `record(` and not `record (`: the call, not the word in a comment. Comment lines
        # are stripped first so a handler cannot be excused by mentioning the audit trail.
        if ! printf '%s\n' "$body" | grep -vE '^[[:space:]]*(//|/\*|\*)' | grep -q 'record('; then
            echo "FAIL: ${handler} is a control-plane write and never calls record()." >&2
            echo "      Every write across tenants must name the operator who made it." >&2
            echo "      See docs/CONTROL_PLANE.md section 5." >&2
            failed=1
        fi
    done

    if [ "$checked" -eq 0 ]; then
        echo "FAIL: no write handlers were found in ${router}; the parse is broken." >&2
        return 1
    fi

    if [ "$failed" -ne 0 ]; then
        return 1
    fi

    echo "    ${checked} write handler(s) audited."
    return 0
}

# A handler that would pass review and fail production: routed as a write, no audit row.
# The script is worth nothing unless it actually rejects this, so it is run every time
# rather than left as a comment claiming it would.
self_test() {
    local dir
    dir=$(mktemp -d)
    trap 'rm -rf "$dir"' RETURN

    mkdir -p "$dir/src"
    cat > "$dir/src/lib.rs" <<'FIXTURE'
pub async fn audited_write(State(ctx): State<Ctx>) -> Response {
    record(&ctx, &actor, "thing.change", "thing", "t_1", None).await;
    ok()
}

pub async fn unaudited_write(State(ctx): State<Ctx>) -> Response {
    // This one only talks about record() in a comment.
    ctx.platform.change_everything().await;
    ok()
}

pub fn routes(ctx: Ctx) -> Router {
    Router::new()
        .route("/thing", post(audited_write))
        .route("/other", put(unaudited_write))
        .with_state(ctx)
}
FIXTURE

    if audit_check "$dir/src" "$dir/src/lib.rs" >/dev/null 2>&1; then
        echo "FAIL: the self-test fixture has an unaudited write and the check passed it." >&2
        echo "      The check is not detecting anything and must be repaired." >&2
        return 1
    fi
    return 0
}

echo "==> Checking the control plane audits every write..."

echo "--> Self-test: an unaudited write must be rejected"
if ! self_test; then
    exit 1
fi
echo "    Rejected, as it must be."

echo "--> ${CRATE}"
if ! audit_check "${CRATE}/src" "$ROUTER"; then
    echo "Control-plane audit check FAILED." >&2
    exit 1
fi

echo "Control-plane audit check PASSED."
exit 0
