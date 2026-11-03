# DESIGN.md: myrtle.moe frontend

Describes the design system as it exists in `src/styles.css` and `src/components/ui` on
2026-11-03. It records what IS there; it does not propose a new brand. When this file and
the code disagree, the code is the truth and this file is stale.

Dial: ENERGY 2 / RHYTHM 2 / MOTION 1

## Identity

An Arknights game-data reference for returning players: operators, stages, enemies, tier
lists, the base planner, gacha history, profiles. People keep it open for long sessions
and read dense tables, so the interface is a tool first. Character comes from the game's
own data colors (rarity, damage type, enemy level) set against a quiet neutral frame and
one red accent, not from decoration.

## Palette

Neutrals are a cool zinc ramp on oklch hue 285 at chroma 0.005, so they never read as
warm grey next to operator art.

| role | light | dark |
|---|---|---|
| background | `oklch(0.985 0.002 285)` | `oklch(0.13 0.005 285)` |
| card / popover | `oklch(1 0 0)` | `oklch(0.17 0.005 285)` |
| muted / accent / secondary | `oklch(0.94 0.005 285)` | `oklch(0.22 0.005 285)` |
| border / input | `oklch(0.88 0.005 285)` | `oklch(0.28 / 0.22 0.005 285)` |
| foreground | `oklch(0.145 0.005 285)` | `oklch(0.97 0.005 285)` |
| muted-foreground | `oklch(0.45 0.01 285)` | `oklch(0.65 0.01 285)` |
| primary / ring | `oklch(0.58 0.22 25)` red | `oklch(0.75 0.15 25)` salmon |

The one accent is the red primary (also `chart-1`, `enemy-boss`, `sidebar-primary`).
Status colors (`info`, `success`, `warning`, `destructive`) come from the Tailwind
blue / emerald / amber / red ramps, at 700 in light and 400 in dark for text.

Game-data colors are content, not brand, and carry meaning players already know:
`--rarity-1..6` (`#b5b5b5`, `#7ef2a3`, `#88c8e3`, `#bcabdb`, `#f7d54c`, `#ff7f27`),
`--dmg-physic / magic / heal / none`, `--enemy-normal / elite / boss`. They are fixed
across themes; on a white card every rarity color measures below 3:1 (1.39 to 2.52), so
in light mode they work as fills, borders and tints. When a rarity has to be TEXT or a
glyph (stars, a coloured count), use `--rarity-N-ink` (`text-rarity-N-ink`): the same hue
darkened to 4.65 to 4.79 on the light page, equal to the plain hue in dark. The ink
tokens are not text-safe on the muted surface (3.97 to 4.17).

Legacy tokens left from the TanStack starter (`--sea-ink-soft`, `--lagoon`,
`--lagoon-deep`, `--palm`, `--surface`, `--bg-base`) survive only for the randomizer and
the leaderboard constants; the rest were deleted on 2026-11-03 with zero users.
`--glow-primary(-intense)` is used by the changelog and profile tabs CSS modules and
`__root.tsx`.

Status text uses the `*-foreground` tokens (`text-success-foreground` 5.14,
`text-warning-foreground` 4.82, `text-info-foreground` 6.55, `text-destructive-foreground`
6.16 on the light page), never a raw `-400` utility, which measures 1.67 to 2.89 there.

## Typography

- `--font-sans` and `--font-heading`: Inter Variable, with a CJK system fallback chain
  so Hangul, kana and Han never drop to a generic face. Body and headings.
- `--font-mono`: Geist Mono Variable. Numbers, ids, and the `.kicker` label.
- `--font-techno`: Orbitron Variable, for RIIC base readouts only.
- `--font-display`: Fraunces Variable, self-hosted from `@fontsource-variable/fraunces`
  (the `opsz` files, upright and italic, the axes the old Google Fonts URL asked for),
  for a few display moments. No font loads from a third party. A new font package also
  needs a `FONT_PKGS` row in `.design-sync/build-css.mjs`.
- `.kicker`: 10.5px Geist Mono, 600, uppercase, 0.12em tracking, muted-foreground. The
  small label over a card. Tracking is dropped under `:lang(ko|ja|zh|th)` by one unlayered
  rule, because the device breaks syllable-block scripts. It is the only eyebrow voice:
  `<Kicker>` renders it, and the settings `SectionLabel` renders `<Kicker>`. `tone="accent"`
  keeps the primary red for the rare label that has to pull the eye.

## Shape, depth, motion

- Radius base `--radius: 0.625rem`; `sm` 0.6x, `md` 0.8x, `lg` 1x, `xl` 1.4x up to `4xl`
  2.6x. Buttons, inputs, toggles and filter chips sit at `lg`. Panels (Card, Alert, the
  card-variant Table) share `2xl`, the radius most panels already rendered at.
- Depth is mostly borders plus `shadow-xs`; elevation shadows are reserved for floating
  layers (popover, dialog, sheet, toast).
- Motion: hover and press transitions, skeleton shimmer, toast success/error keyframes,
  the router progress bar. No scroll reveals, parallax or decorative loops. One base
  `prefers-reduced-motion` rule collapses every transition and animation except spinners.
- Focus: one base `:focus-visible` outline, 2px solid `--ring` at a 2px offset (4.59 light,
  8.06 dark); kit controls that draw their own ring set `outline-none`.
- Touch: kit controls reach 44px under `pointer-coarse` through a centred `::after` hit
  area (pattern in `ui/button.tsx`), without changing the desktop layout. A tooltip opens on
  a tap when its trigger has no action of its own; a trigger that is a working button or
  link keeps its click and shows the hint on hover and focus only.
- Disabled: kit items blocked by Base UI (menu, select, combobox, tabs, slider, number
  field, drawer checkbox and radio rows, calendar days) keep pointer events, so the base
  `cursor: not-allowed` shows. Sidebar items keep `aria-disabled:pointer-events-none`,
  since `aria-disabled` alone stops nothing.

## Layout

- `.page-shell`: one gutter (`--page-gutter` 0.75rem, 1rem from `sm`) shared with the
  header, a per-page `--page-max` (default 1400px), padding-block 1.75rem 5rem.
- `.page-gutter` for pages that open with a full-bleed hero; `.page-bleed` to pull a
  sticky bar or a scroll rail out to the content edge.
- `--site-header-height`: 57px below `sm`, 65px from `sm`.
- `.scroll-shadows`: CSS-only edge shadows on a scroller, so a cut line reads as "more".
- Body reserves `--music-bar-reserve` while the music player bar is mounted.

## Components

`src/components/ui` is a shadcn-style kit on Base UI with `cva` and `tailwind-merge`.
Icons are lucide-react. Charts are recharts. Strings live in `*.messages.ts` next to the
component, never inline.

## Do not touch from a design pass

OG card renderers (the operator card is pixel-locked), dynchar / pixi / chibi / spine
viewers, story reader plates, PDF / EPUB export renderers, `src/types/generated/**`,
`routeTree.gen.ts`.
