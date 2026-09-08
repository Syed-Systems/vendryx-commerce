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

## The test

An HTTP handler exists to move data between a client and a service. Its service arrives in
its `State(...)` binding. **A handler that builds a response body and never uses that
binding did not consult anything, so whatever it returned was written in the source.**

Passing the binding to a same-file helper counts as using it -- `get_provider_health`
reaches the ledger through `collect_ledger(&ctx)`, and that is real data access.

### Why not "did it await anything?"

That was the original test and it inspected almost nothing. Every control-plane handler
opens with `actor(&ctx, &headers).await` for authentication and `record(...).await` for the
audit row, so *every handler in `stheno-api-platform` passed regardless of whether it
touched data* -- including `replay_dead_letter_webhook`, which audits a replay it never
performs. Measured across the API layer, 361 of 531 handlers were skipped on the `.await`
rule alone and the guard's real inspection surface was four handlers.

The authentication and audit preamble is therefore stripped before the test runs. Proving
who the caller is says nothing about where the answer came from.
"""

import glob
import os
import re
import sys

# Authentication and audit helpers. Every control-plane handler calls these, and calling
# them is not evidence that the response body came from anywhere. Stripped before testing.
PREAMBLE = ("actor", "deny", "record", "require_operator")

# Genuinely static answers. Each needs a reason that is a DECISION, not a restatement
# of the handler name. These are the only entries that may claim to be correct as constants.
CONSTANT = {
    ("localisation.rs", "translation_settings"): "translatable fields are a model property, not data",
    ("store.rs", "list_notifications"): "notification feed is not implemented; empty is honest",
    ("views.rs", "active_configuration"): "null means 'use defaults', which the UI handles",
    ("views.rs", "create_configuration"): "view configuration acknowledgment before saved view table migration",
    ("views.rs", "list_columns"): "column definitions are static per entity",
    ("views.rs", "list_configurations"): "no saved views yet; empty is the true answer",
    ("views.rs", "list_entities"): "column definitions are static per entity",
    ("views.rs", "set_active_configuration"): "active view confirmation before saved view table migration",
}

# Known stubs: they answer from literals and are NOT correct. This is a debt ledger, not a
# pardon. It exists because 159 stubs were previously filed under CONSTANT with reasons that
# merely restated the handler name ("create_gift_card" -> "gift card creation"), which made
# deliberate constants and unimplemented endpoints indistinguishable.
#
# Rules:
#   * The gate PASSES with entries here, and prints the count. Debt is tracked, not blocking.
#   * The count may not GROW without sign-off. CI compares against STUB_BUDGET below.
#   * Wiring a handler means DELETING its line here. That is the only way the number falls.
UNIMPLEMENTED = {
    # catalog.rs — catalog association endpoint; waits on the link write path
    ("catalog.rs", "batch_product_image_variants"): "stub",
    ("catalog.rs", "batch_product_variants_inventory"): "stub",
    ("catalog.rs", "batch_variant_images"): "stub",
    ("catalog.rs", "confirm_product_import"): "stub",
    ("catalog.rs", "create_variant_inventory_item"): "stub",
    ("catalog.rs", "delete_variant_inventory_item"): "stub",
    ("catalog.rs", "export_products"): "stub",
    ("catalog.rs", "import_products"): "stub",
    ("catalog.rs", "update_variant_inventory_item"): "stub",
    # config.rs — gift cards; no gift_card table exists
    ("config.rs", "create_gift_card"): "stub",
    ("config.rs", "get_gift_card"): "stub",
    ("config.rs", "list_gift_card_orders"): "stub",
    ("config.rs", "list_gift_cards"): "stub",
    ("config.rs", "update_gift_card"): "stub",
    # fulfillment_settings.rs — fulfillment options and service zones; waits on FulfillmentService methods
    ("fulfillment_settings.rs", "add_location_fulfillment_set"): "stub",
    ("fulfillment_settings.rs", "get_fulfillment_set_service_zone"): "stub",
    ("fulfillment_settings.rs", "list_fulfillment_provider_options"): "stub",
    ("fulfillment_settings.rs", "manage_location_fulfillment_providers"): "stub",
    ("fulfillment_settings.rs", "manage_location_sales_channels"): "stub",
    ("fulfillment_settings.rs", "update_fulfillment_set_service_zone"): "stub",
    # lib.rs — platform/admin surface stub
    ("lib.rs", "run_workflow"): "stub",
    ("lib.rs", "step_failure"): "stub",
    ("lib.rs", "step_success"): "stub",
    ("lib.rs", "subscribe_workflow"): "stub",
    ("lib.rs", "subscribe_workflow_transaction"): "stub",
    # orders.rs — order sub-resource; mounted for Admin API verb parity, never calls OrderService
    ("orders.rs", "add_claim_inbound_items"): "stub",
    ("orders.rs", "add_claim_inbound_shipping_method"): "stub",
    ("orders.rs", "add_claim_items"): "stub",
    ("orders.rs", "add_claim_outbound_items"): "stub",
    ("orders.rs", "add_claim_outbound_shipping_method"): "stub",
    ("orders.rs", "add_draft_order_edit_items"): "stub",
    ("orders.rs", "add_draft_order_edit_promotions"): "stub",
    ("orders.rs", "add_draft_order_edit_shipping_methods"): "stub",
    ("orders.rs", "add_exchange_inbound_items"): "stub",
    ("orders.rs", "add_exchange_inbound_shipping_method"): "stub",
    ("orders.rs", "add_exchange_outbound_items"): "stub",
    ("orders.rs", "add_exchange_outbound_shipping_method"): "stub",
    ("orders.rs", "add_order_credit_lines"): "stub",
    ("orders.rs", "add_order_edit_items"): "stub",
    ("orders.rs", "add_order_edit_shipping_method"): "stub",
    ("orders.rs", "add_return_request_items"): "stub",
    ("orders.rs", "add_return_shipping_method"): "stub",
    ("orders.rs", "add_store_credit_account_credit"): "stub",
    ("orders.rs", "authorize_order_payment_sessions"): "stub",
    ("orders.rs", "cancel_claim"): "stub",
    ("orders.rs", "cancel_claim_request"): "stub",
    ("orders.rs", "cancel_exchange"): "stub",
    ("orders.rs", "cancel_exchange_request"): "stub",
    ("orders.rs", "cancel_order_transfer"): "stub",
    ("orders.rs", "cancel_return"): "stub",
    ("orders.rs", "cancel_return_receive"): "stub",
    ("orders.rs", "cancel_return_request"): "stub",
    ("orders.rs", "confirm_draft_order_edit"): "stub",
    ("orders.rs", "confirm_return_receive"): "stub",
    ("orders.rs", "confirm_return_request"): "stub",
    ("orders.rs", "convert_draft_order_to_order"): "stub",
    ("orders.rs", "create_draft_order_edit"): "stub",
    ("orders.rs", "create_payment_collection_session"): "stub",
    ("orders.rs", "create_store_credit_account"): "stub",
    ("orders.rs", "delete_claim_inbound_item"): "stub",
    ("orders.rs", "delete_claim_inbound_shipping_method"): "stub",
    ("orders.rs", "delete_claim_item"): "stub",
    ("orders.rs", "delete_claim_outbound_item"): "stub",
    ("orders.rs", "delete_claim_outbound_shipping_method"): "stub",
    ("orders.rs", "delete_dismiss_return_item"): "stub",
    ("orders.rs", "delete_draft_order"): "stub",
    ("orders.rs", "delete_draft_order_edit"): "stub",
    ("orders.rs", "delete_draft_order_edit_item_action"): "stub",
    ("orders.rs", "delete_draft_order_edit_promotions"): "stub",
    ("orders.rs", "delete_draft_order_edit_shipping_method_action"): "stub",
    ("orders.rs", "delete_draft_order_edit_shipping_method_by_id"): "stub",
    ("orders.rs", "delete_exchange_inbound_item"): "stub",
    ("orders.rs", "delete_exchange_inbound_shipping_method"): "stub",
    ("orders.rs", "delete_exchange_outbound_item"): "stub",
    ("orders.rs", "delete_exchange_outbound_shipping_method"): "stub",
    ("orders.rs", "delete_order_edit_item_action"): "stub",
    ("orders.rs", "delete_order_edit_shipping_method"): "stub",
    ("orders.rs", "delete_payment_collection"): "stub",
    ("orders.rs", "delete_receive_return_item"): "stub",
    ("orders.rs", "delete_return_request_item"): "stub",
    ("orders.rs", "delete_return_shipping_method"): "stub",
    ("orders.rs", "dismiss_return_items"): "stub",
    ("orders.rs", "export_orders"): "stub",
    ("orders.rs", "get_order_changes"): "stub",
    ("orders.rs", "get_order_shipping_options"): "stub",
    ("orders.rs", "get_return"): "stub",
    ("orders.rs", "get_store_credit_account"): "stub",
    ("orders.rs", "guest_order_transfer"): "stub",
    ("orders.rs", "list_payment_providers"): "stub",
    ("orders.rs", "list_payments"): "stub",
    ("orders.rs", "list_store_credit_account_transactions"): "stub",
    ("orders.rs", "list_store_credit_accounts"): "stub",
    ("orders.rs", "receive_return_items"): "stub",
    ("orders.rs", "request_claim"): "stub",
    ("orders.rs", "request_draft_order_edit"): "stub",
    ("orders.rs", "request_exchange"): "stub",
    ("orders.rs", "update_claim_inbound_item"): "stub",
    ("orders.rs", "update_claim_inbound_shipping_method"): "stub",
    ("orders.rs", "update_claim_item"): "stub",
    ("orders.rs", "update_claim_outbound_item"): "stub",
    ("orders.rs", "update_claim_outbound_shipping_method"): "stub",
    ("orders.rs", "update_dismiss_return_item"): "stub",
    ("orders.rs", "update_draft_order"): "stub",
    ("orders.rs", "update_draft_order_edit_item_action"): "stub",
    ("orders.rs", "update_draft_order_edit_original_item"): "stub",
    ("orders.rs", "update_draft_order_edit_shipping_method_action"): "stub",
    ("orders.rs", "update_draft_order_edit_shipping_method_by_id"): "stub",
    ("orders.rs", "update_exchange_inbound_item"): "stub",
    ("orders.rs", "update_exchange_inbound_shipping_method"): "stub",
    ("orders.rs", "update_exchange_outbound_item"): "stub",
    ("orders.rs", "update_exchange_outbound_shipping_method"): "stub",
    ("orders.rs", "update_order_edit_item_action"): "stub",
    ("orders.rs", "update_order_edit_original_item"): "stub",
    ("orders.rs", "update_order_edit_shipping_method"): "stub",
    ("orders.rs", "update_receive_return_item"): "stub",
    ("orders.rs", "update_return"): "stub",
    ("orders.rs", "update_return_request_item"): "stub",
    ("orders.rs", "update_return_shipping_method"): "stub",
    # promotions.rs — promotion sub-resource; waits on PromotionService methods
    ("promotions.rs", "batch_promotion_buy_rules"): "stub",
    ("promotions.rs", "batch_promotion_rules"): "stub",
    ("promotions.rs", "batch_promotion_target_rules"): "stub",
    ("promotions.rs", "list_rule_attribute_options"): "stub",
    ("promotions.rs", "list_rule_value_options"): "stub",
    # store.rs — store-surface resource with no storage behind it yet
    ("store.rs", "admin_search"): "stub",
    ("store.rs", "batch_property_labels"): "stub",
    ("store.rs", "batch_translation_settings"): "stub",
    ("store.rs", "batch_translations"): "stub",
    ("store.rs", "create_property_label"): "stub",
    ("store.rs", "delete_property_label"): "stub",
    ("store.rs", "get_index_details"): "stub",
    ("store.rs", "get_notification"): "stub",
    ("store.rs", "get_property_label"): "stub",
    ("store.rs", "list_plugins"): "stub",
    ("store.rs", "list_property_labels"): "stub",
    ("store.rs", "list_translatable_entities"): "stub",
    ("store.rs", "sync_index"): "stub",
    ("store.rs", "update_property_label"): "stub",
    # views.rs — saved views + widget layout; waits on the saved-view table migration
    ("views.rs", "delete_configuration"): "stub",
    ("views.rs", "delete_layout_configuration"): "stub",
    ("views.rs", "get_configuration"): "stub",
    ("views.rs", "get_layout_configuration"): "stub",
    ("views.rs", "list_layout_configurations"): "stub",
    ("views.rs", "set_layout_configuration"): "stub",
    ("views.rs", "update_configuration"): "stub",
}

# The number of known stubs at the time this ledger was written. It is a ceiling, not a
# target: lower it as handlers are wired, and never raise it without saying why in the commit.
STUB_BUDGET = 144


HANDLER = re.compile(r"pub async fn (\w+)\s*\(")
STATE_BINDING = re.compile(r"State\((\w+)\)")
ANY_FN = re.compile(r"^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)", re.M)


def _brace_match(src, start):
    depth = 0
    for i in range(start, len(src)):
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                return i
    return len(src) - 1


def handler_bodies(src):
    """Yields (name, signature, body) for each `pub async fn`, brace-matched."""
    for match in HANDLER.finditer(src):
        start = src.find("{", match.end())
        if start == -1:
            continue
        end = _brace_match(src, start)
        yield match.group(1), src[match.end():start], src[start:end + 1]


def all_fn_names(src):
    """Every fn in the file, so helper indirection can be recognised."""
    names = set()
    for match in ANY_FN.finditer(src):
        names.add(match.group(1))
    return names


def strip_preamble(body):
    for helper in PREAMBLE:
        body = re.sub(helper + r"\s*\([^)]*\)", "", body)
    return body


IDENT = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")


def _uses(text, var):
    """True when `var` is used as an identifier: followed by a dot or a paren, or
    passed by reference.

    Written with plain scanning rather than a regex. The check needs a word boundary,
    and a mis-escaped one silently matches nothing -- which would make this whole gate
    report success on every file. Scanning cannot be mis-escaped.

    Whitespace after the identifier is skipped, because method chains wrap: a handler
    that reads its state across two lines is still reading it. Checking only the very
    next character marked every multi-line chain in this codebase as never-used.
    """
    i = text.find(var)
    while i != -1:
        before = text[i - 1] if i else " "
        j = i + len(var)
        while j < len(text) and text[j].isspace():
            j += 1
        after = text[j:j + 1]
        if before not in IDENT and (after in (".", ")") or before == "&"):
            return True
        i = text.find(var, i + 1)
    return False


def consults_a_service(signature, body, file_fns):
    """True when the handler uses its own State binding for something other than auth."""
    state = STATE_BINDING.search(signature)
    if not state or state.group(1).startswith("_"):
        return False
    var = state.group(1)
    if _uses(strip_preamble(body), var):
        return True
    # Indirection: handed to a helper defined in this file, which does the reading.
    for helper in file_fns:
        if helper in PREAMBLE:
            continue
        call = helper + "("
        i = body.find(call)
        while i != -1:
            args = body[i + len(call):i + len(call) + 64]
            if args.lstrip().lstrip("&").startswith(var):
                return True
            i = body.find(call, i + 1)
    return False


def selftest():
    """Guards the two ways this gate silently stops working.

    Both have already happened once. A predicate that never matches turns the whole
    check into a green light over an unread codebase, and nothing else would notice.
    """
    fns = {"collect_ledger", "actor", "record"}

    # 1. A wrapped method chain is a real read. Reading only the next character after
    #    the binding marked every multi-line handler in the codebase as fabricated.
    nl = chr(10)
    wrapped = ("{ let x = ctx" + nl + "        .order_service" + nl +
               "        .list_orders().await; }")
    assert consults_a_service("State(ctx): State<C>", wrapped, fns), "wrapped chain"

    # 2. Auth and audit are not data access. Every control-plane handler awaits both,
    #    which is why the previous rule inspected 4 handlers out of 531.
    auth_only = ('{ let Some(a) = actor(&ctx, &h).await else { return deny(); };'
                 ' record(&ctx, &a, "webhook.replay").await;'
                 ' Json(json!({"status": "replayed"})) }')
    assert not consults_a_service("State(ctx): State<C>", auth_only, fns), "auth-only"

    # 3. Indirection through a same-file helper is a real read.
    via_helper = '{ let l = collect_ledger(&ctx).await; Json(json!({"n": l.len()})) }'
    assert consults_a_service("State(ctx): State<C>", via_helper, fns), "helper"

    # 4. An unused binding is not a read.
    unused = '{ Json(json!({"providers": ["stripe"]})) }'
    assert not consults_a_service("State(ctx): State<C>", unused, fns), "unused"

    print("selftest OK: 4 predicate cases")
    return 0


def main():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    os.chdir(root)

    # The predicate is the gate. If it stops matching, every file reads as clean and this
    # script reports success over an unread codebase -- which is the exact failure it was
    # written to catch, pointed at itself. Cheap enough to run every time.
    try:
        selftest()
    except AssertionError as exc:
        print(f"ERROR: predicate selftest failed ({exc}). The gate cannot be trusted.")
        return 1

    offenders = []
    stubs = []
    for path in sorted(glob.glob("crates/api/*/src/*.rs")):
        src = open(path, encoding="utf8").read()
        basename = os.path.basename(path)
        file_fns = all_fn_names(src)
        for name, signature, body in handler_bodies(src):
            if "json!(" not in body:
                continue
            if consults_a_service(signature, body, file_fns):
                continue
            key = (basename, name)
            if key in CONSTANT:
                continue
            if key in UNIMPLEMENTED:
                stubs.append((path.replace("\\", "/"), name))
                continue
            offenders.append((path.replace("\\", "/"), name))

    if offenders:
        print("ERROR: API handlers answering with invented data (AGENTS.md: no stub handlers)")
        print()
        for path, name in offenders:
            print(f"  {path}  ->  {name}")
        print()
        print("Each builds a JSON body without ever using its State binding, so the data it")
        print("returns is written in the source. Either wire it to its domain service, or:")
        print("  * if it is genuinely a constant, add it to CONSTANT with a reason that is a")
        print("    decision -- not a restatement of the handler name;")
        print("  * if it is an unimplemented stub, add it to UNIMPLEMENTED and raise")
        print("    STUB_BUDGET in the same commit, saying why the debt grew.")
        return 1

    if len(stubs) > STUB_BUDGET:
        print(f"ERROR: known-stub count rose to {len(stubs)}, above STUB_BUDGET={STUB_BUDGET}.")
        print("Raising the budget is a spec change (TASKS.md L3) and needs sign-off.")
        return 1

    print(f"Fabricated-handler check PASSED: {len(CONSTANT)} deliberate constants, "
          f"{len(stubs)} known stubs (budget {STUB_BUDGET}).")
    if len(stubs) < STUB_BUDGET:
        print(f"  {STUB_BUDGET - len(stubs)} stub(s) wired since the budget was set -- "
              f"lower STUB_BUDGET to {len(stubs)} to lock the gain in.")
    return 0


if __name__ == "__main__":
    if "--selftest" in sys.argv:
        sys.exit(selftest())
    sys.exit(main())
