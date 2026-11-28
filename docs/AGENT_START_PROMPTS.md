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
| R | Reviewing a finished phase PR before you merge it | A different agent from the one that wrote it |

## How to run the whole project

The full order is in [STOREFRONT_THEME_ENGINE_PLAN.md §1.7](STOREFRONT_THEME_ENGINE_PLAN.md).
In practice, repeat this loop until the board in plan §4 shows every task `done`:

1. Open the task board (plan §4). Find the lowest-numbered `todo` task whose dependencies are
   `done`.
2. If it is an `O` task, it is yours: do it, then set it to `done` on the board (or tell an
   agent to).
3. Otherwise start a **new** agent session in `D:\Syed Systems\Store` and paste prompt A. One
   session per task. Run one engine agent at a time unless two tasks have no dependency on each
   other and you give each agent its own git worktree.
4. Read its report. If it says `blocked`, answer the question in the report, put your answer
   in the board row, and start a new session with prompt A again.
5. When every task of a phase is `done`, start a session with a **different** agent and paste
   prompt R. Fix what it finds with prompt A ("Fix review finding <n> on the Phase <n> PR").
   Then merge the phase PR yourself.
6. Themes in AI Studio run alongside all of this with prompt B (or C after T1.10).

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
   Tasks named O1-O7 belong to the owner: never do one. If your task depends on an open O task,
   set it to `blocked: waiting for O<n>` and stop.
3. Read that task's full entry in plan section 5. It names every file you may create or edit,
   every function and type you must use, the tests to write, and the exact commands that prove
   it works.
4. Follow plan section 1.6 "Rules that prevent guessing" without exception. In short: never
   invent a name, a file or data; never weaken or skip a test or gate; never claim a result you
   did not see; stay inside the task's file list; if anything is missing or unclear, set the task
   to `blocked: <reason>` on the board, push that, and stop.
5. Work on the phase branch, never a branch of your own: there is ONE branch and ONE PR per
   phase (plan 1.1 step 3). Claim the task first (plan 1.1 step 4), then do it, then finish it
   (plan 1.3).
6. Write code on this machine, but run every build, clippy and test ON SPACE with
   scripts/build-on-space.ps1 (plan 1.5). Never run cargo build/test/clippy, npm run build or
   Playwright locally. If Space is unreachable, stop and report it; do not build locally.
7. Finishing: paste the real output lines from Space into the phase PR description under your
   task's heading, tick each acceptance criterion with its evidence, commit as
   `<TASK-ID>: <task title>`, set the board to `done (<commit sha>)`, push. Mark the PR ready
   only if this was the last task of the phase and CI is green.
8. Do not start a second task. Report back: the task id, the commit sha, the Space commands you
   ran and their final output lines, and anything you set to blocked.

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
   test), fix it, and reproduce the check on Space (never on this machine) before pushing (commands in the plan,
   section 1.4 and 1.5).
3. Forbidden: deleting or loosening an assertion, #[ignore], test.skip, widening an allow-list,
   adding an audit ignore, or editing a gate script to pass - unless the check itself is
   provably wrong, in which case explain why in the commit message.
4. One fix per commit, message says what broke and why. Push, wait for the run, repeat.
5. Report: each failure, its root cause, the fix commit, and the final `gh pr checks` output.
```

---

## R. Phase PR review

```
Review the Phase <n> pull request <PR link> of the Vendryx commerce repository. You did not
write it. Do not edit code; produce a review.

1. Read AGENTS.md, docs/STOREFRONT_THEME_ENGINE_PLAN.md sections 1 and 2, and the Phase <n>
   tasks in section 5.
2. For every task in the phase: compare the diff with the task's Create/Edit lists (any file
   changed outside them is a finding), check each acceptance criterion against the evidence in
   the PR description, and confirm each named test exists with that exact name.
3. Re-run the verification yourself on Space (plan section 1.5), never locally. Quote the final
   output lines. A criterion whose evidence you could not reproduce is a finding.
4. Look specifically for: a price, total, discount or stock value computed outside the Store
   API; hardcoded product names, prices, ratings or statistics; a weakened, skipped or deleted
   test; a cookie without HttpOnly, Secure and SameSite; a cached response that contains
   shopper data; `|safe` on loader data; an external script or URL not in
   docs/payment-page-scripts.md; any AI product name in the repository.
5. Output: a numbered list of findings, each with file:line, what is wrong, and the smallest
   fix; then a verdict line: "APPROVE" or "CHANGES REQUIRED (<count>)".
```

