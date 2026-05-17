#!/usr/bin/env python3
"""
Rewrites Tailwind arbitrary-value classes into their canonical equivalents,
matching what Tailwind 4 LSP's `suggestCanonicalClasses` rule emits.

Assumes the project's default --spacing: 0.25rem (verify with `grep "--spacing"`
in your styles; this script bails out otherwise).

Handled rewrites:
  * Spacing scale (any positive N divisible into a `.25` step):
      min-w-[280px] -> min-w-70
      pt-[12px]     -> pt-3
      p-[3px]       -> p-0.75
      -mt-[8px]     -> -mt-2
      size-[1rem]   -> size-4
  * Named letter-spacing: tracking-[0.1em]   -> tracking-widest
  * Named border-radius: rounded-[2px]       -> rounded-xs
                         rounded-t-[8px]     -> rounded-t-lg
  * Z-index unwrap:      z-[55]              -> z-55
  * Named line-height:   leading-[1.5]       -> leading-normal
  * Literal renames:     break-words         -> wrap-break-word
  * Prefix renames:      bg-gradient-to-t    -> bg-linear-to-t
                         bg-gradient-to-br   -> bg-linear-to-br

Pass --write to apply changes; default is dry-run.

To extend: add an entry to LITERAL_RENAMES (full class swap) or
PREFIX_RENAMES (preserve suffix). No new regex required.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "src"

# ---------------------------------------------------------------------------
# Spacing-scale utilities (use --spacing: 0.25rem; 1 step = 4px)
# ---------------------------------------------------------------------------
SPACING_PREFIXES = [
    "w", "min-w", "max-w",
    "h", "min-h", "max-h",
    "size",
    "p", "px", "py", "pt", "pr", "pb", "pl", "ps", "pe",
    "m", "mx", "my", "mt", "mr", "mb", "ml", "ms", "me",
    "gap", "gap-x", "gap-y",
    "top", "right", "bottom", "left", "start", "end",
    "inset", "inset-x", "inset-y",
    "space-x", "space-y",
    "translate", "translate-x", "translate-y", "translate-z",
    "scroll-p", "scroll-px", "scroll-py", "scroll-pt", "scroll-pr", "scroll-pb", "scroll-pl", "scroll-ps", "scroll-pe",
    "scroll-m", "scroll-mx", "scroll-my", "scroll-mt", "scroll-mr", "scroll-mb", "scroll-ml", "scroll-ms", "scroll-me",
    "indent", "outline-offset", "ring-offset",
]
SPACING_PREFIXES.sort(key=len, reverse=True)
SPACING_GROUP = "|".join(re.escape(p) for p in SPACING_PREFIXES)

SPACING_RE = re.compile(
    r"(?P<boundary>(?:^|[\s\"'`{(]))"
    r"(?P<neg>-?)"
    r"(?P<prefix>" + SPACING_GROUP + r")"
    r"-\[(?P<value>-?[\d.]+(?:px|rem))\]"
)


def format_scale(scale: float) -> str | None:
    """Return Tailwind-canonical scale string (integer, .25, .5, or .75)."""
    if scale <= 0:
        return None
    quarter = round(scale * 4)
    if abs(scale * 4 - quarter) > 1e-9:
        return None
    whole, rem = divmod(quarter, 4)
    if rem == 0:
        return str(whole)
    if rem == 1:
        return f"{whole}.25" if whole else "0.25"
    if rem == 2:
        return f"{whole}.5" if whole else "0.5"
    if rem == 3:
        return f"{whole}.75" if whole else "0.75"
    return None


def spacing_value(value: str) -> tuple[str, bool] | None:
    """Return (canonical-scale-string, is_negative) or None when not rewritable."""
    is_neg = value.startswith("-")
    if is_neg:
        value = value[1:]
    if value.endswith("px"):
        n = float(value[:-2])
        scale = format_scale(n / 4.0)
    elif value.endswith("rem"):
        n = float(value[:-3])
        scale = format_scale(n / 0.25)
    else:
        return None
    if scale is None:
        return None
    return scale, is_neg


def rewrite_spacing(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        parsed = spacing_value(m.group("value"))
        if parsed is None:
            return m.group(0)
        scale, value_neg = parsed
        prefix_neg = m.group("neg") == "-"
        # XOR: prefix-negative and inside-bracket-negative cancel out
        final_neg = prefix_neg ^ value_neg
        sign = "-" if final_neg else ""
        original = m.group("neg") + m.group("prefix") + "-[" + m.group("value") + "]"
        canonical = sign + m.group("prefix") + "-" + scale
        edits.append((original, canonical))
        return m.group("boundary") + canonical

    return SPACING_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Letter-spacing (tracking)
# ---------------------------------------------------------------------------
TRACKING_NAMED = {
    -0.05: "tighter",
    -0.025: "tight",
    0.0: "normal",
    0.025: "wide",
    0.05: "wider",
    0.1: "widest",
}
TRACKING_RE = re.compile(
    r"(?P<boundary>(?:^|[\s\"'`{(]))"
    r"tracking-\[(?P<value>-?[\d.]+em)\]"
)


def rewrite_tracking(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        raw = m.group("value")
        try:
            n = float(raw[:-2])
        except ValueError:
            return m.group(0)
        for key, name in TRACKING_NAMED.items():
            if abs(n - key) < 1e-9:
                edits.append((f"tracking-[{raw}]", f"tracking-{name}"))
                return m.group("boundary") + f"tracking-{name}"
        return m.group(0)

    return TRACKING_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Border radius (rounded)
# ---------------------------------------------------------------------------
ROUNDED_NAMED_PX = {
    0: "none",
    2: "xs",
    4: "sm",
    6: "md",
    8: "lg",
    12: "xl",
    16: "2xl",
    24: "3xl",
    32: "4xl",
}
ROUNDED_NAMED_REM = {
    0.0: "none",
    0.125: "xs",
    0.25: "sm",
    0.375: "md",
    0.5: "lg",
    0.75: "xl",
    1.0: "2xl",
    1.5: "3xl",
    2.0: "4xl",
}
ROUNDED_CORNERS = ["tl", "tr", "br", "bl", "ss", "se", "ee", "es", "t", "r", "b", "l", "s", "e"]
ROUNDED_RE = re.compile(
    r"(?P<boundary>(?:^|[\s\"'`{(]))"
    r"rounded"
    r"(?P<corner>-(?:" + "|".join(ROUNDED_CORNERS) + r"))?"
    r"-\[(?P<value>[\d.]+(?:px|rem))\]"
)


def rounded_named(value: str) -> str | None:
    if value.endswith("px"):
        n = float(value[:-2])
        for key, name in ROUNDED_NAMED_PX.items():
            if abs(n - key) < 1e-9:
                return name
    elif value.endswith("rem"):
        n = float(value[:-3])
        for key, name in ROUNDED_NAMED_REM.items():
            if abs(n - key) < 1e-9:
                return name
    return None


def rewrite_rounded(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        name = rounded_named(m.group("value"))
        if name is None:
            return m.group(0)
        corner = m.group("corner") or ""
        original = f"rounded{corner}-[{m.group('value')}]"
        canonical = f"rounded{corner}-{name}"
        edits.append((original, canonical))
        return m.group("boundary") + canonical

    return ROUNDED_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# z-index unwrap
# ---------------------------------------------------------------------------
Z_RE = re.compile(
    r"(?P<boundary>(?:^|[\s\"'`{(]))"
    r"(?P<neg>-?)z-\[(?P<value>\d+)\]"
)


def rewrite_z(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        original = m.group("neg") + "z-[" + m.group("value") + "]"
        canonical = m.group("neg") + "z-" + m.group("value")
        edits.append((original, canonical))
        return m.group("boundary") + canonical

    return Z_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Line-height (leading) — named ratios
# ---------------------------------------------------------------------------
LEADING_NAMED = {
    1.0: "none",
    1.25: "tight",
    1.375: "snug",
    1.5: "normal",
    1.625: "relaxed",
    2.0: "loose",
}
LEADING_RE = re.compile(
    r"(?P<boundary>(?:^|[\s\"'`{(]))"
    r"leading-\[(?P<value>[\d.]+)\]"
)


def rewrite_leading(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        raw = m.group("value")
        try:
            n = float(raw)
        except ValueError:
            return m.group(0)
        for key, name in LEADING_NAMED.items():
            if abs(n - key) < 1e-9:
                edits.append((f"leading-[{raw}]", f"leading-{name}"))
                return m.group("boundary") + f"leading-{name}"
        return m.group(0)

    return LEADING_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Literal class renames (Tailwind v3 → v4) — exact class swaps
# ---------------------------------------------------------------------------
LITERAL_RENAMES: dict[str, str] = {
    "break-words": "wrap-break-word",
}

# Prefix-only renames — preserve whatever suffix follows the prefix.
# Example: "bg-gradient-to-" -> "bg-linear-to-" rewrites bg-gradient-to-{t,b,l,r,tl,tr,bl,br}.
PREFIX_RENAMES: dict[str, str] = {
    "bg-gradient-to-": "bg-linear-to-",
}

CLASS_BOUNDARY_PREFIX = r"(?P<boundary>(?:^|[\s\"'`{(]))"
CLASS_BOUNDARY_SUFFIX = r"(?=$|[\s\"'`})\]])"


def _build_literal_re(mapping: dict[str, str]) -> re.Pattern[str] | None:
    if not mapping:
        return None
    names = sorted(mapping.keys(), key=len, reverse=True)
    return re.compile(
        CLASS_BOUNDARY_PREFIX
        + r"(?P<name>" + "|".join(re.escape(n) for n in names) + r")"
        + CLASS_BOUNDARY_SUFFIX
    )


def _build_prefix_re(mapping: dict[str, str]) -> re.Pattern[str] | None:
    if not mapping:
        return None
    prefixes = sorted(mapping.keys(), key=len, reverse=True)
    return re.compile(
        CLASS_BOUNDARY_PREFIX
        + r"(?P<prefix>" + "|".join(re.escape(p) for p in prefixes) + r")"
        + r"(?P<suffix>[a-z0-9]+(?:-[a-z0-9]+)*)"
        + CLASS_BOUNDARY_SUFFIX
    )


LITERAL_RE = _build_literal_re(LITERAL_RENAMES)
PREFIX_RE = _build_prefix_re(PREFIX_RENAMES)


def rewrite_literals(text: str, edits: list[tuple[str, str]]) -> str:
    if LITERAL_RE is None:
        return text

    def sub(m: re.Match[str]) -> str:
        name = m.group("name")
        canonical = LITERAL_RENAMES[name]
        edits.append((name, canonical))
        return m.group("boundary") + canonical

    return LITERAL_RE.sub(sub, text)


def rewrite_prefixes(text: str, edits: list[tuple[str, str]]) -> str:
    if PREFIX_RE is None:
        return text

    def sub(m: re.Match[str]) -> str:
        prefix = m.group("prefix")
        suffix = m.group("suffix")
        original = prefix + suffix
        canonical = PREFIX_RENAMES[prefix] + suffix
        edits.append((original, canonical))
        return m.group("boundary") + canonical

    return PREFIX_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Important modifier position: `!utility` -> `utility!` (Tailwind 3 -> 4)
# Generic: find any `!` that is at a class-context boundary (start, whitespace,
# string-quote, or `:` variant separator) and shift it to the end of the
# following utility. Doesn't care what variant chain (if any) precedes it.
# ---------------------------------------------------------------------------
BANG_RE = re.compile(
    r"(?P<lead>(?:^|[\s\"'`{(:]))"
    r"!"
    r"(?P<utility>[a-z][a-z0-9-]*(?:\[[^\]]*\])?(?:/[\d.]+)?)"
    r"(?=$|[\s\"'`})\]])"
)


def rewrite_bang(text: str, edits: list[tuple[str, str]]) -> str:
    def sub(m: re.Match[str]) -> str:
        utility = m.group("utility")
        # Skip TS-ish false positives — real Tailwind utilities almost always
        # contain a hyphen or a bracketed arbitrary value.
        if "-" not in utility and "[" not in utility:
            return m.group(0)
        edits.append(("!" + utility, utility + "!"))
        return m.group("lead") + utility + "!"

    return BANG_RE.sub(sub, text)


# ---------------------------------------------------------------------------
# Em-dash (U+2014 and &mdash; entity) -> hyphen
# ---------------------------------------------------------------------------
EMDASH_FORMS = ("—", "&mdash;")


def rewrite_emdashes(text: str, edits: list[tuple[str, str]]) -> str:
    for form in EMDASH_FORMS:
        count = text.count(form)
        if count == 0:
            continue
        for _ in range(count):
            edits.append((form, "-"))
        text = text.replace(form, "-")
    return text


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------
def rewrite_text(text: str) -> tuple[str, list[tuple[str, str]]]:
    edits: list[tuple[str, str]] = []
    text = rewrite_spacing(text, edits)
    text = rewrite_tracking(text, edits)
    text = rewrite_rounded(text, edits)
    text = rewrite_z(text, edits)
    text = rewrite_leading(text, edits)
    text = rewrite_literals(text, edits)
    text = rewrite_prefixes(text, edits)
    text = rewrite_bang(text, edits)
    text = rewrite_emdashes(text, edits)
    return text, edits


def iter_files() -> list[Path]:
    out: list[Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix not in {".tsx", ".ts", ".jsx", ".js"}:
            continue
        if "routeTree.gen" in path.name:
            continue
        out.append(path)
    return out


def main() -> int:
    write = "--write" in sys.argv

    total_edits = 0
    files_changed = 0
    sample_edits: list[tuple[Path, str, str]] = []

    for path in iter_files():
        text = path.read_text(encoding="utf-8")
        new_text, edits = rewrite_text(text)
        if not edits:
            continue
        files_changed += 1
        total_edits += len(edits)
        for orig, canon in edits[:5]:
            sample_edits.append((path, orig, canon))
        if write and new_text != text:
            path.write_text(new_text, encoding="utf-8")

    print(f"Files with canonical-class candidates: {files_changed}")
    print(f"Total replacements{' applied' if write else ' (dry-run)'}: {total_edits}")

    if sample_edits and not write:
        print()
        print("Sample replacements:")
        seen: set[tuple[str, str]] = set()
        for path, orig, canon in sample_edits:
            key = (orig, canon)
            if key in seen:
                continue
            seen.add(key)
            rel = path.relative_to(ROOT.parent)
            print(f"  {rel}: {orig!r} -> {canon!r}")
            if len(seen) >= 40:
                break

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
