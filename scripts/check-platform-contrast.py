#!/usr/bin/env python3
"""Fails if apps/platform's token layer drops below WCAG AA.

Written because the first pass did. apple-design's published palette uses #86868b for
secondary text and #ff3b30 for errors; on white those are 3.62:1 and 3.55:1, and AA
wants 4.5:1 for body text. A re-token pass cannot be eyeballed for this — the numbers
have to be computed, and computed again the next time someone tunes a colour.

Parses the CSS rather than restating it. A hardcoded copy of the palette would drift
from the file it claims to check, which is the failure mode `ALL_FEATURES` already has
a doc comment about.

Usage: python scripts/check-platform-contrast.py
"""

import pathlib
import re
import sys

CSS = pathlib.Path(__file__).resolve().parent.parent / "apps/platform/src/index.css"

# (foreground token, background token, minimum ratio). 3.0 is the AA floor for
# disabled/incidental text, 4.5 for anything a person is expected to read.
PAIRS = [
    ("--fg-base", "--bg-base", 4.5),
    ("--fg-base", "--bg-subtle", 4.5),
    ("--fg-subtle", "--bg-base", 4.5),
    ("--fg-subtle", "--bg-subtle", 4.5),
    ("--fg-muted", "--bg-base", 4.5),
    ("--fg-muted", "--bg-subtle", 4.5),
    ("--fg-muted", "--bg-component", 4.5),
    ("--fg-error", "--bg-base", 4.5),
    ("--fg-interactive", "--bg-base", 4.5),
    ("--fg-on-color", "--bg-interactive", 4.5),
    ("--fg-on-color", "--button-inverted", 4.5),
    ("--tag-neutral-text", "--tag-neutral-bg", 4.5),
    ("--fg-disabled", "--bg-base", 3.0),
]


def blocks(text):
    """The `:root` and `.dark` declaration blocks, as {token: raw value}."""
    out = {}
    for selector in (":root", r"\.dark"):
        match = re.search(selector + r"\s*\{(.*?)\n\}", text, re.S)
        if not match:
            sys.exit(f"{CSS}: no {selector} block")
        out[selector.replace("\\", "")] = dict(
            re.findall(r"(--[\w-]+)\s*:\s*([^;]+);", match.group(1))
        )
    return out


def resolve(token, scope, root, seen=()):
    """Follows var() chains, falling back to :root the way the cascade does."""
    if token in seen:
        sys.exit(f"circular variable reference at {token}")
    value = scope.get(token, root.get(token))
    if value is None:
        sys.exit(f"{token} is not defined in :root or .dark")
    value = value.strip()
    ref = re.fullmatch(r"var\((--[\w-]+)\)", value)
    return resolve(ref.group(1), scope, root, seen + (token,)) if ref else value


def luminance(hex_colour):
    hex_colour = hex_colour.lstrip("#")
    if len(hex_colour) != 6:
        sys.exit(f"only 6-digit hex is checkable, got {hex_colour!r}")
    channels = []
    for i in (0, 2, 4):
        c = int(hex_colour[i : i + 2], 16) / 255
        channels.append(c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4)
    r, g, b = channels
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def ratio(fg, bg):
    a, b = luminance(fg), luminance(bg)
    hi, lo = max(a, b), min(a, b)
    return (hi + 0.05) / (lo + 0.05)


def main():
    parsed = blocks(CSS.read_text(encoding="utf-8"))
    root = parsed[":root"]
    failures = []
    for theme, scope in (("light", root), ("dark", parsed[".dark"])):
        for fg_token, bg_token, need in PAIRS:
            fg = resolve(fg_token, scope, root)
            bg = resolve(bg_token, scope, root)
            # rgba() surfaces compose against whatever is behind them; skip rather than
            # pretend to measure one. None of the pairs above should hit this.
            if not fg.startswith("#") or not bg.startswith("#"):
                continue
            got = ratio(fg, bg)
            status = "ok  " if got >= need else "FAIL"
            print(f"  {status} {theme:5} {fg_token} on {bg_token}: {got:.2f} (need {need})")
            if got < need:
                failures.append(f"{theme} {fg_token} on {bg_token}: {got:.2f} < {need}")

    if failures:
        print("\napps/platform contrast is below WCAG AA:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        sys.exit(1)
    print("\napps/platform token layer meets WCAG AA in both themes.")


if __name__ == "__main__":
    main()
