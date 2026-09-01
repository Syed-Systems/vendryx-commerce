#!/usr/bin/env python3
"""Generates `openapi-store.yaml` from the routes the engine actually mounts.

Why generated and not written: the Storefront API is a published contract with several
independent consumers — each themed storefront lives in its own repository and codes
against the document rather than reading this tree. A hand-maintained spec is a second
copy of the truth, and this codebase has already paid for second copies three times over:
handlers that returned invented JSON, tests that asserted values the handlers had stopped
producing, and a route table nothing compared against the client. A stale contract shared
by four storefronts is worse than none, because every consumer inherits the same wrong
assumption and none of them can see it.

So the spec is derived from `.route(...)` declarations in `crates/api/stheno-api-store`
and `crates/stheno-server`. Run with `--check` in CI: it regenerates into memory and fails
if the committed file has drifted, which makes "add a route, forget the doc" a build
failure instead of a support ticket from another repo.

Descriptions are read from the `///` doc comment on each handler, so the prose lives next
to the code it describes and moves when that moves.
"""

import argparse
import collections
import glob
import io
import os
import re
import sys

ROUTE = re.compile(r'\.route\(\s*"([^"]+)"\s*,')
VERB = re.compile(r"(?:^|[\s.:(])(get|post|put|patch|delete)\s*\(\s*([a-z_]+(?:::[a-z_]+)?)")
DOC = re.compile(r"((?:^///.*\n)+)^pub async fn (\w+)", re.M)

SURFACE = "/store/"


def mounted_routes(root):
    """Every (path, verb, handler) the store surface mounts."""
    found = []
    files = glob.glob(os.path.join(root, "crates/api/stheno-api-store/src/*.rs"))
    files += [os.path.join(root, "crates/stheno-server/src/lib.rs")]
    for f in files:
        src = io.open(f, encoding="utf8").read()
        i = 0
        while True:
            m = ROUTE.search(src, i)
            if not m:
                break
            path = m.group(1)
            j, depth = m.end(), 1
            while j < len(src) and depth > 0:
                depth += (src[j] == "(") - (src[j] == ")")
                j += 1
            body = src[m.end() : j - 1]
            if path.startswith(SURFACE) or path == SURFACE.rstrip("/"):
                for verb, handler in VERB.findall(body):
                    found.append((path, verb.upper(), handler.split("::")[-1]))
            i = j
    return sorted(set(found))


def handler_docs(root):
    """handler name -> its `///` doc comment, first line only."""
    docs = {}
    for f in glob.glob(os.path.join(root, "crates/api/stheno-api-store/src/*.rs")):
        src = io.open(f, encoding="utf8").read()
        for block, name in DOC.findall(src):
            lines = [
                ln.strip().lstrip("/").strip()
                for ln in block.strip().split("\n")
                if ln.strip().startswith("///")
            ]
            summary = ""
            for ln in lines:
                if ln:
                    summary = ln
                    break
            if summary and not summary.startswith(("GET ", "POST ", "DELETE ", "PUT ")):
                docs[name] = summary
    return docs


def to_yaml(routes, docs):
    """Emits OpenAPI 3.0 by hand — the shape is small and fixed, and this keeps the
    script free of a dependency the rest of the repo does not already carry."""
    by_path = collections.OrderedDict()
    for path, verb, handler in routes:
        by_path.setdefault(path, []).append((verb, handler))

    out = []
    out.append("# GENERATED FILE - do not edit by hand.")
    out.append("# Regenerate with: python scripts/generate-openapi-store.py")
    out.append("# CI runs this with --check, so an edit here is reverted by the next build.")
    out.append("openapi: 3.0.0")
    out.append("info:")
    out.append("  title: Stheno Storefront API")
    out.append("  version: 1.0.0")
    out.append("  description: |")
    out.append("    The contract every storefront codes against.")
    out.append("")
    out.append("    Two things are not visible in the paths below and every consumer needs them:")
    out.append("")
    out.append("    - The tenant is resolved from the request Host. One deployment serves")
    out.append("      many merchants; nothing in the path or body selects one.")
    out.append("    - The customer account surface is per-tenant. A merchant without the")
    out.append("      `buyer_accounts` entitlement has no /store/customers routes at all, and")
    out.append("      they answer 404 rather than 403. Read GET /store/config first and adapt;")
    out.append("      a storefront that assumes accounts exist will break on some tenants.")
    out.append("servers:")
    out.append("  - url: http://localhost:9000")
    out.append("paths:")

    for path, ops in by_path.items():
        out.append(f"  {path}:")
        params = re.findall(r"\{(\w+)\}", path)
        for verb, handler in sorted(ops):
            out.append(f"    {verb.lower()}:")
            out.append(f"      operationId: {handler}")
            summary = docs.get(handler, handler.replace("_", " ").capitalize())
            out.append("      summary: " + yaml_scalar(summary))
            out.append("      tags: [" + path.split("/")[2] + "]")
            if params:
                out.append("      parameters:")
                for p in params:
                    out.append(f"        - name: {p}")
                    out.append("          in: path")
                    out.append("          required: true")
                    out.append("          schema: { type: string }")
            out.append("      responses:")
            out.append("        '200': { description: Success }")
    return "\n".join(out) + "\n"


def yaml_scalar(text):
    text = text.replace("`", "").strip()
    if any(ch in text for ch in ":#{}[]&*!|>'\"%@"):
        return '"' + text.replace('"', "'") + '"'
    return text


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="fail if the committed file drifted")
    args = ap.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    routes = mounted_routes(root)
    docs = handler_docs(root)
    rendered = to_yaml(routes, docs)
    target = os.path.join(root, "openapi-store.yaml")

    if args.check:
        if not os.path.exists(target):
            print("ERROR: openapi-store.yaml is missing. Run scripts/generate-openapi-store.py")
            return 1
        current = io.open(target, encoding="utf8").read()
        if current != rendered:
            print("ERROR: openapi-store.yaml is out of date with the mounted routes.")
            print("A storefront in another repository is reading this file. Regenerate it:")
            print("    python scripts/generate-openapi-store.py")
            return 1
        print(f"openapi-store.yaml is current ({len(routes)} endpoints).")
        return 0

    io.open(target, "w", encoding="utf8", newline="\n").write(rendered)
    paths = len({r[0] for r in routes})
    print(f"Wrote openapi-store.yaml: {len(routes)} endpoints across {paths} paths.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
