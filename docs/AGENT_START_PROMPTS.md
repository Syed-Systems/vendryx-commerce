# Agent start prompts

Copy-paste prompts for starting an agent on this repository. Each one is complete on its own:
paste it as the agent's first message, change nothing except the text in `<angle brackets>`.

The rules behind them live in [AGENTS.md](../AGENTS.md) and
[STOREFRONT_THEME_ENGINE_PLAN.md](STOREFRONT_THEME_ENGINE_PLAN.md) §1. The prompts point there
instead of repeating them, so updating the plan updates every prompt.

| Prompt | Use it for | Tool |
|---|---|---|
| A | Building the theme engine, one task at a time | Any coding agent with repo access |
| B | Designing a new theme **before** task T1.10 is done | Google AI Studio, Antigravity |
| C | Designing a new theme **after** task T1.10 is done | Google AI Studio, Antigravity |
| D | Fixing a red CI run | Any coding agent with repo access |

Which of B or C applies: open the task board (plan §4). If T1.10 says `done`, use C.

---

## A. Engine task

```
You are working on the Vendryx commerce repository (Rust engine, admin app, storefront).
Your job is exactly one task from the storefront theme engine plan, start to finish.

1. Pull main. Read, in this order and completely: AGENTS.md, docs/DECISIONS.md ADR-038,
   then docs/STOREFRONT_THEME_ENGINE_PLAN.md sections 1, 2 and 4.
2. Pick the task by the rule in plan section 1.1 (lowest-numbered `todo` whose dependencies are
   all `done`). <Optional: "Do task T1.2." to assign one directly.>
3. Read that task's full entry in plan section 5. It names every file you may create or edit,
   every function and type you must use, the tests to write, and the exact commands that prove
   it works.
4. Follow plan section 1.6 "Rules that prevent guessing" without exception. In short: never
   invent a name, a file or data; never weaken or skip a test or gate; never claim a result you
   did not see; stay inside the task's file list; if anything is missing or unclear, set the task
   to `blocked: <reason>` on the board, push that, and stop.
5. Claim the task first (plan 1.1 step 4), then do it, then finish it (plan 1.3): run the task's
   Verify commands and the global gates (plan 1.4), paste the real output lines into the PR
   description, tick each acceptance criterion with its evidence, set the board to
   `done (<PR link>)`.
6. Do not start a second task. Report back: the task id, the PR link, the commands you ran and
   their final output lines, and anything you set to blocked.

Never put an AI product or company name in a branch, commit, comment, PR or document.
No Co-Authored-By trailers, no "generated with" footers, no emojis.
```

---

## B. Theme design, before the engine renders themes (now)

The engine cannot render minijinja templates yet, so the design tool keeps building the React
preview it is good at. The rules below make that preview convert to a minijinja theme
mechanically later (plan Phase 3), with no redesign.

```
Design a storefront theme for the Vendryx commerce platform. You are a designer and tokenizer
only. The platform supplies all data and all commerce logic; you supply the look.

Deliver one self-contained folder: design-source/<theme-id>-<YYYY-MM-DD>/
  - a React + Vite preview app of the storefront (home, collection/listing, product, cart,
    account, about), as you normally build it
  - theme.json at the folder root, in exactly this shape:
    { "schema_version": 1, "id": "<theme-id>", "name": "<Theme Name>",
      "tokens": [ { "id": "...", "type": "...", "default": ..., "label": "..." } ],
      "sections": { "index": [ { "type": "hero", "settings": [ ... ] } ] } }
    Token types allowed: color, range (min, max, unit), select (options), font, text
    (max_length required), richtext, image, url, boolean. Every token has a merchant-readable
    "label".
  - mock-data.ts: the ONLY file that may contain product names, prices, images or any other
    sample content. Clearly fake values only ("Sample Product", 10.00).
  - README.md: one paragraph on the design idea, and the list of pages built.

Hard rules (a design that breaks one is rejected):
1. Every colour, font, radius, shadow and spacing value is a token in theme.json. The CSS
   reads them only as custom properties named after the token id: var(--theme-<token-id>).
   No #hex, rgb(), hsl(), named colour, font-family name, box-shadow or border-radius value
   anywhere outside theme.json.
2. Every piece of visible copy (headings, button labels, banner text) is a text or richtext
   setting in theme.json, with the copy as its "default". Components read it from settings.
3. Components read data only through these shapes, imported from mock-data.ts:
   product { title, handle, description, thumbnail, images[], options[], variants[] }
   variant { title, calculated_price?: { calculated_amount, currency_code } }
   collection { title, handle, products[] }
   cart { items[], subtotal, total, currency_code }
   store { name, logo, currency }
   Format every price through one function formatMoney(amount, currency_code). A variant with
   no calculated_price shows "Unavailable", never 0.
4. Never invent facts: no ratings, reviews, "N sold", "trusted by N", stock counts, delivery
   promises or statistics, not even in mock-data.ts. If the design needs one, leave it out.
5. Do not build checkout, payment or sign-in forms. Put a clearly labelled placeholder box
   ("Checkout - provided by the platform") where they go.
6. Animations, drawers, galleries and WebGL go in their own files under src/islands/, so they
   can become standalone scripts. The page must be readable with them switched off.
7. No external URLs: no CDN fonts, no remote images, no analytics. Fonts are referenced by a
   key in a "font" token, never by URL.
8. Default colours meet WCAG 2.2 AA contrast (4.5:1 for body text).
9. Write nothing outside your folder.
```

---

## C. Theme design, after T1.10 is done

Once the engine renders themes and `scripts/check-theme-pack.sh` exists, the design tool writes
the final theme directly. Use the prompt in [THEME_AUTHORING.md](THEME_AUTHORING.md) §8, and
give the tool the whole of THEME_AUTHORING.md plus the `themes/_starter/` folder as context.
Task T5.1 finalizes that prompt.

---

## D. Fixing red CI

```
CI on <branch or PR link> is failing. Make it green without weakening anything.

1. Read AGENTS.md. Run `gh pr checks <PR number>` and `gh run view <run id> --log-failed`.
   Work on the EARLIEST failing stage first: 1 (static, typecheck) -> 2 (Rust tests, web tests)
   -> 3 (end-to-end) -> 4 (container image, supply chain). Later stages are skipped while an
   earlier one fails, so a green stage 2 can reveal a new failure in stage 3; that is expected.
2. For each failure, quote the exact failing line, find the root cause in the code (not in the
   test), fix it, and reproduce the check locally before pushing (commands in the plan,
   section 1.4 and 1.5).
3. Forbidden: deleting or loosening an assertion, #[ignore], test.skip, widening an allow-list,
   adding an audit ignore, or editing a gate script to pass - unless the check itself is
   provably wrong, in which case explain why in the commit message.
4. One fix per commit, message says what broke and why. Push, wait for the run, repeat.
5. Report: each failure, its root cause, the fix commit, and the final `gh pr checks` output.
```
