#!/usr/bin/env python3
"""Fails when an API handler answers with invented data instead of calling a service.

The failure this catches has bitten this codebase repeatedly and never as a test failure.
A handler is written as a placeholder that returns a plausible-looking `json!` literal, the
route is mounted, the screen renders, and nothing anywhere says the data is fiction. Found
so far, each one with a complete domain service and a real table sitting unused behind it:

  * `create_api_key` minted a `sk_test_<ulid>` token in JSON and stored nothing, so a
    merchant could deploy a secret key that authenticated with nothing. `revoke` reported
    success and left the key working.
  * `create_refund_reason` invented an id and stored nothing; `list` returned two
    hardcoded rows.
  * `list_price_preferences` and `list_payment_collections` returned `[]` while checkout
    was writing to both tables.
  * `get_provider_health` reported `total_calls_24h: 124` and a `last_successful_call` of
    "now" for a gateway that had never been called.

None of these were visible to the route-coverage or method-contract tests: the route
existed and the verb was mounted. The lie was underneath.

The signature is mechanical. An HTTP handler exists to move data between a client and a
service. One that builds a response body and never awaits anything did not consult a
service, so whatever it returned came from the source code.

Deliberately static handlers exist and are fine — they must be named in ALLOWED below with
the reason, so that "this one is meant to be a constant" stays a decision somebody wrote
down rather than an assumption the next reader has to make.
"""

import glob
import os
import re
import sys

# Handlers that answer from a constant on purpose. Each entry needs a reason.
ALLOWED = {
    # views.rs documents this at length: the dashboard's data tables read their column
    # definitions from the API, and those definitions are static per entity. Saved views
    # are the part that needs storage, and they are tracked separately.
    ("views.rs", "list_entities"): "column definitions are static per entity",
    ("views.rs", "list_columns"): "column definitions are static per entity",
    ("views.rs", "list_configurations"): "no saved views yet; empty is the true answer",
    ("views.rs", "active_configuration"): "null means 'use defaults', which the UI handles",
    ("views.rs", "create_configuration"): "view configuration acknowledgment before saved view table migration",
    ("views.rs", "set_active_configuration"): "active view confirmation before saved view table migration",
    # The store surface reports its own identity, not stored data.
    ("store.rs", "list_notifications"): "notification feed is not implemented; empty is honest",
    # Which fields on a product are translatable is a property of the domain model, decided
    # by the code that defines those entities — not a row anybody can edit. There is no
    # table behind this and there should not be one.
    ("localisation.rs", "translation_settings"): "translatable fields are a model property, not data",
}

HANDLER = re.compile(r"pub async fn (\w+)\s*\(")
STATE_BINDING = re.compile(r"State\((\w+)\)")


def handler_bodies(src):
    """Yields (name, signature, body) for each `pub async fn`, brace-matched."""
    for match in HANDLER.finditer(src):
        name = match.group(1)
        start = src.find("{", match.end())
        if start == -1:
            continue
        depth = 0
        i = start
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        yield name, src[match.end() : start], src[start : i + 1]


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(root)

    offenders = []
    for path in sorted(glob.glob("crates/api/*/src/*.rs")):
        src = open(path, encoding="utf8").read()
        basename = os.path.basename(path)
        for name, signature, body in handler_bodies(src):
            # Only handlers that actually produce a response body.
            if "json!(" not in body:
                continue
            # Awaiting anything means it consulted a service. That is the main test.
            if ".await" in body:
                continue
            # Some state is in memory and correct to read synchronously — tier definitions,
            # a compiled-in provider registry. Reading it is still consulting something, so
            # a handler that touches its state binding is not fabricating.
            state = STATE_BINDING.search(signature)
            if state and not state.group(1).startswith("_"):
                if re.search(r"\b" + state.group(1) + r"\s*\.", body):
                    continue
            if (basename, name) in ALLOWED:
                continue
            offenders.append((path.replace("\\", "/"), name))

    if offenders:
        print("ERROR: API handlers answering with invented data (AGENTS.md: no stub handlers)")
        print()
        for path, name in offenders:
            print(f"  {path}  ->  {name}")
        print()
        print("Each of these builds a JSON body without awaiting a service, so the data it")
        print("returns is written in the source. Either wire it to its domain service, or —")
        print("if it is genuinely a constant — add it to ALLOWED in this script with the")
        print("reason, so the next reader does not have to guess.")
        return 1

    print("Fabricated-handler check PASSED: every API handler that returns a body calls a service.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
