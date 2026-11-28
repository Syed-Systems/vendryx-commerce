#!/usr/bin/env bash
# P29-10: relative links between documents resolve.
#
# This repository routes decisions through documents that reference each other — an ADR
# is cited from a task row, a task row from a status page, a status page from a README.
# A broken link is not cosmetic here: the reason a thing was built the way it was becomes
# unreachable, and the next person rebuilds the decision from scratch.
#
# Checks relative links only. External URLs are not fetched: a network call would make
# this gate fail on someone else's outage.
set -euo pipefail

# Markdown files that are part of *our* decision record.
#
# Vendored and generated trees have their own link conventions, are not ours to fix, and
# a gate that fails on them is a gate people learn to ignore. `.agents` is vendored agent
# tooling whose links resolve outside this repo; `apps/admin` is the forked Medusa
# dashboard, whose CHANGELOG ships literal `](undefined)` links.
mapfile -t FILES < <(
    find . -name '*.md' \
        -not -path './node_modules/*' \
        -not -path './target/*' \
        -not -path './*/node_modules/*' \
        -not -path './apps/*/dist/*' \
        -not -path './apps/admin/*' \
        -not -path './.agents/*' \
        -not -path './docs-site/*' \
        | sort
)

echo "==> Checking relative links in ${#FILES[@]} document(s)..."

FAILED=0
CHECKED=0

for file in "${FILES[@]}"; do
    dir=$(dirname "$file")

    # `[text](target)` — the target only. Fenced code is not excluded on purpose: a
    # command in a code block that names a file is as worth checking as a prose link.
    while IFS= read -r target; do
        # Leave external links, anchors-in-place and mailto to their owners.
        case "$target" in
            http://*|https://*|mailto:*|'#'*|'') continue ;;
        esac

        # `FILE.md#anchor` — the file half is what has to exist. Anchor text is generated
        # from headings by the renderer and is not reliably checkable from here.
        path="${target%%#*}"
        [ -z "$path" ] && continue

        CHECKED=$((CHECKED + 1))

        if [ ! -e "${dir}/${path}" ]; then
            echo "FAIL: ${file} links to ${target}, which does not exist" >&2
            FAILED=1
        fi
    done < <(grep -oE '\]\([^)]+\)' "$file" | sed -E 's/^\]\((.*)\)$/\1/')
done

if [ "$CHECKED" -eq 0 ]; then
    echo "FAIL: no links were checked; the parse is broken." >&2
    exit 1
fi

if [ "$FAILED" -ne 0 ]; then
    echo "Docs link check FAILED." >&2
    exit 1
fi

echo "Docs link check PASSED: ${CHECKED} relative link(s) resolve."
exit 0
