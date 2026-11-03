# DESIGN.md: myrtle.moe frontend

Describes the design system as it exists in `src/styles.css` and `src/components/ui` on
2026-11-04. It records what IS there; it does not propose a new brand. When this file and
the code disagree, the code is the truth and this file is stale.

Dial: ENERGY 2 / RHYTHM 2 / MOTION 1

## Identity

An Arknights game-data reference for returning players: operators, stages, enemies, tier
lists, the base planner, gacha history, profiles. People keep it open for long sessions
and read dense tables, so the interface is a tool first. Character comes from the game's
own data colors (rarity, damage type, enemy level) set against a quiet neutral frame and
one red accent, not from decoration.

## Palette

### Neutrals

A cool zinc ramp on oklch hue 285 at chroma 0.005, so it never reads as warm grey next to
operator art.

| role | light | dark |
|---|---|---|
| background | `oklch(0.985 0.002 285)` | `oklch(0.13 0.005 285)` |
| card / popover | `oklch(1 0 0)` | `oklch(0.17 0.005 285)` |
| muted / accent / secondary | `oklch(0.94 0.005 285)` | `oklch(0.22 0.005 285)` |
| border / input | `oklch(0.88 0.005 285)` | border `oklch(0.28 0.005 285)`, input `oklch(0.22 0.005 285)` |
| foreground | `oklch(0.145 0.005 285)` | `oklch(0.97 0.005 285)` |
| muted-foreground | `oklch(0.45 0.01 285)` | `oklch(0.65 0.01 285)` |
| subtle-foreground | `oklch(0.52 0.01 285)` | `oklch(0.62 0.01 285)` |
| primary / primary-ink / ring | `oklch(0.58 0.22 25)` red, 4.59 page / 4.79 card | `oklch(0.75 0.15 25)` salmon, 8.49 page / 8.07 card |
| primary-hover | primary 88% + black, label 6.08 | primary 88% + white, label 9.46 |

### Accent

The one accent is the red primary (also `chart-1`, `enemy-boss`, `sidebar-primary`).
`--primary` is the fill. `text-primary` and `decoration-primary` read `--primary-ink`, and
`ring-primary` / `outline-primary` read `--ring` (namespace keys in `@theme inline`); `bg`,
`border` and `fill` stay on the fill. By default all three are the same red, so the split
only matters for a picked accent: it keeps its fill while text and ring get variants that
clear 4.5 and 3 on the page.

`lib/theme/accent-kit.ts` is the one implementation (the pre-paint script in `__root.tsx`
embeds its `toString()`). A custom hex stays the fill and its label is black or white by
the higher ratio. A preset hue steps L from the default (down in light, up in dark, about
0.54 to 0.58 in light, chroma clamped to sRGB) until it clears 4.5 on page and card and
under its label, and 3 on muted. Hover and pressed fills use `bg-primary-hover`, never
`bg-primary/90` (which measured 4.16 page / 4.14 card).

### Status

`info`, `success`, `warning` fills are Tailwind blue / emerald / amber 500;
`--destructive` is its own oklch red (`oklch(0.577 0.245 27.325)` light,
`oklch(0.396 0.141 25.723)` dark). Status text uses the `*-foreground` tokens, which are
the 700 step in light and the 400 step in dark (red for destructive):
`text-success-foreground` 5.14, `text-warning-foreground` 4.82, `text-info-foreground` 6.55,
`text-destructive-foreground` 6.16 on the light page; `text-destructive-foreground` 6.61 on
the dark card, where `text-destructive` measured 1.80 on its own tint. Never a raw `-300` /
`-400` / `-500` utility for status text, which measures 1.52 to 3.82 there.

### Data colours and their inks

Game-data colors are content, not brand, and carry meaning players already know:
`--rarity-1..6` (`#b5b5b5`, `#7ef2a3`, `#88c8e3`, `#bcabdb`, `#f7d54c`, `#ff7f27`),
`--dmg-physic / magic / heal / none`, `--enemy-normal / elite / boss`, plus the stage-group
and banner-status hues. On a white card every rarity color measures below 3:1 (1.39 to
2.52), so in light mode the plain hues work as fills, borders, dots and tints only.

When a data colour has to be TEXT or a glyph (stars, a coloured count, a label), use its
`-ink` twin (`text-*-ink`): the same oklch hue, lightness lowered in light mode, chroma
trimmed only to stay in sRGB. Dark mode uses the plain hue, which already reads.

- Rarity: `--rarity-N-ink` measures 4.65 to 4.77 on the light page and 4.85 to 4.98 on the
  card; dark is the plain hue (7.58 to 13.28 on the dark card). In code, `rarityInk(n)`
  (`lib/utils`) returns `var(--rarity-n-ink)` for text and glyphs; `rarityStarColor` /
  `RARITY_HEX(_MUTED)` stay for fills, borders and rings.
- The other data colours clear 4.6 on background and card and 4.5 on a 15% tint of their
  own plain hue. Ratios are light background / card / own 15% tint, then dark card / tint:

| token | light | measured light | dark | measured dark |
|---|---|---|---|---|
| `--enemy-normal-ink` | `oklch(0.53 0.01 285)` | 5.07 / 5.29 / 4.57 | `--enemy-normal` | 8.58 / 6.70 |
| `--enemy-elite-ink` | `oklch(0.53 0.111 75)` | 5.16 / 5.39 / 4.74 | `--enemy-elite` | 7.55 / 6.06 |
| `--enemy-boss-ink` | `oklch(0.53 0.214 25)` | 5.66 / 5.91 / 4.66 | `--enemy-boss` | 8.07 / 6.43 |
| `--dmg-physic-ink` | `oklch(0.54 0.093 65)` | 4.96 / 5.18 / 4.61 | `--dmg-physic` | 8.08 / 6.39 |
| `--dmg-magic-ink` | `oklch(0.54 0.13 304.5)` | 5.13 / 5.36 / 4.72 | `--dmg-magic` | 7.36 / 5.91 |
| `--dmg-heal-ink` | `oklch(0.53 0.109 157.5)` | 4.79 / 5.00 / 4.60 | `--dmg-heal` | 10.68 / 8.04 |
| `--dmg-none-ink` | `oklch(0.52 0 0)` | 5.27 / 5.51 / 4.72 | `--dmg-none` | 5.54 / 4.63 |
| `--stage-events-ink` | `oklch(0.54 0.123 62.9)` | 5.02 / 5.24 / 4.73 | `#f7a452` | 9.46 / 7.31 |
| `--stage-ra-ink` | `oklch(0.52 0.125 156.3)` | 4.96 / 5.18 / 4.62 | `#5bbf86` | 8.42 / 6.63 |
| `--stage-sss-ink` | `oklch(0.53 0.145 260.2)` | 5.14 / 5.37 / 4.61 | `#5a8fe8` | 5.95 / 4.94 |
| `--stage-cc-ink` | `oklch(0.54 0.115 72.5)` | 4.96 / 5.18 / 4.64 | `#e8a23c` | 8.80 / 6.89 |
| `--banner-active-ink` | `oklch(0.52 0.163 145)` | 4.92 / 5.14 / 4.67 | `oklch(0.78 0.18 145)` | 10.18 / 7.77 |
| `--banner-upcoming-ink` | `oklch(0.53 0.096 220)` | 4.90 / 5.11 / 4.57 | `oklch(0.78 0.16 220)` | 10.18 / 7.86 |
| `--banner-returning-ink` | `oklch(0.55 0.12 300)` | 4.88 / 5.09 / 4.61 | `oklch(0.78 0.12 300)` | 9.20 / 7.11 |
| `--featured-six-ink` | `oklch(0.54 0.111 80)` | 4.92 / 5.14 / 4.61 | `oklch(0.78 0.18 80)` | 9.40 / 7.30 |

Code reaches them through `LEVEL_TOKENS[..].ink` / `DAMAGE_TOKENS[..].ink`
(`enemies/list/impl/tokens.ts`), `ENEMY_LEVEL_INK` (stage detail), `STAGE_GROUP_INK`
(`lib/registry/stage-groups.ts`; IS reuses `--dmg-magic-ink`, Paradox `--rarity-4-ink`,
Annihilation `--destructive-foreground`) and `BANNER_STATUS_INK` (gacha).

No ink is text-safe on the muted surface at every hue (rarity 3.97 to 4.17, the others
4.07 to 4.62). Data-coloured text on its own tint fails too (1.66 to 3.96), so a chip or
badge that sits on a tone tint keeps the tone on its border and fill and takes
`foreground` text (see Components); a chip on an art scrim takes white.

Game rich-text spans in operator descriptions (`<@ba.kw>`, `<$ba.magic>`, `vup`, `vdown`,
`rem`) take `--game-kw-ink` / `--game-vup-ink` / `--game-vdown-ink` / `--game-rem-ink`
through `inkForTag` (`operators/detail/impl/description.ts`). Light: `oklch(0.5 0.12 194.3)`
5.14 / 5.37 / 4.50, `oklch(0.52 0.12 261.3)` 5.34 / 5.57 / 4.67, `oklch(0.53 0.12 24.7)`
5.37 / 5.61 / 4.70, `oklch(0.53 0.12 70.8)` 5.18 / 5.41 / 4.53 (background / card /
muted; the game's #27e8e7 measured 1.5 there). Dark keeps the game's own colours, 5.83 to
13.21 on background, card and muted. The story reader keeps `colorForTag` unchanged.

Free-form user colours (tier-list flairs) never become text. A label painted ON a user
colour (a tier row's pill) takes black or white from `readableTextColor`, whichever has the
higher WCAG ratio against it.

### Text steps

Three steps: foreground, muted-foreground for secondary text, and `text-subtle-foreground`
for the quietest meta (counts, ids, timestamps, kickers, separators, empty-slot
placeholders, placeholder text). Subtle measures 5.29 / 5.52 / 4.63 on background / card /
muted in light and 5.51 / 5.24 / 4.75 in dark. Text never takes an alpha step
(`text-muted-foreground/70`, `/50`): over the page those measured 2.32 to 4.47, and 128
sites failed AA before the token replaced them. `/90` still clears 4.5 on every surface and
is tolerated; anything quieter uses subtle. On art or a translucent scrim, measure against
the real backdrop instead.

### Leftovers

Legacy tokens from the TanStack starter (`--sea-ink-soft`, `--lagoon`, `--lagoon-deep`,
`--palm`, `--surface`, `--bg-base`) survive only for the randomizer and the leaderboard
constants; the rest were deleted on 2026-11-03 with zero users. `--glow-primary(-intense)`
is still defined and rewritten by the accent kit, but no stylesheet or component reads it.

## Typography

- `--font-sans` and `--font-heading`: Inter Variable, with a CJK fallback chain
  (`--font-cjk`) so Hangul, kana and Han never drop to a generic face. Body and headings.
- `--font-mono`: Geist Mono Variable. Numbers, ids, and the `.kicker` label.
- `--font-techno`: Orbitron Variable, for RIIC base readouts only.
- `--font-display`: Fraunces Variable, self-hosted from `@fontsource-variable/fraunces`
  (the `opsz` files, upright and italic), for a few display moments. No font loads from a
  third party. A new font package also needs a `FONT_PKGS` row in
  `.design-sync/build-css.mjs`.
- `.kicker`: 10.5px Geist Mono, 600, uppercase, 0.12em tracking, muted-foreground. The
  small label over a card, and the only eyebrow voice: `<Kicker>` renders it, and the
  settings `SectionLabel` renders `<Kicker>`. `tone="accent"` keeps the primary red for the
  rare label that has to pull the eye. `<Kicker>` carries no margin: most callers sit it
  in a flex row or a gap stack, so the caller that heads a block adds `mb-2.5`. It labels
  a section; a name, a date or a live count is meta text, not a kicker.
- Tracking is dropped under `:lang(ko|ja|zh|th)` by one unlayered rule (on `tracking-*`,
  `.uppercase` and `.kicker`), because the device breaks syllable-block scripts.

## Shape, depth, motion

- Radius base `--radius: 0.625rem`; `sm` 0.6x, `md` 0.8x, `lg` 1x, `xl` 1.4x, `2xl` 1.8x
  up to `4xl` 2.6x. Buttons, inputs, toggles and filter chips sit at `lg`. Panels (Card,
  Alert, the card-variant Table) share `2xl`.
- Depth is mostly borders plus `shadow-xs`; elevation shadows are reserved for floating
  layers (popover, dialog, sheet, toast).
- Motion: hover and press transitions, skeleton shimmer, toast success/error keyframes,
  the router progress bar. No scroll reveals, parallax or decorative loops. The one kept
  loop is the stage map's tile effects (`stages/detail/map/impl/effects.css`), which copy
  the game's animated tiles, so the motion is data that tells tile kinds apart. One base
  `prefers-reduced-motion` rule collapses every transition and animation, those tiles
  included; spinners keep turning, slower.
- Focus: the mark is always solid `--ring` at 2px, never an alpha step (`ring-ring/24`
  measured 1.46, `/60` 2.70, a 50% outline 2.29 light / 2.80 dark, all under the 3:1 a focus
  mark needs). The base `:focus-visible` rule draws `outline: 2px solid var(--ring)` at a
  2px offset; a kit control that draws its own ring sets `outline-none`. Standalone kit
  controls (button, toggle, badge, switch, checkbox, radio, slider, menu switch, scroll-area,
  accordion trigger, tab panel) draw `ring-2 ring-ring ring-offset-2
  ring-offset-background`, the same geometry. Exceptions, each for a reason:
  - full-width rows and anything inside a scroller (drawer rows, sidebar rows, tab
    triggers in their scrolling strips) use `inset-ring-2` so the ring is not clipped;
  - fields (input, select, textarea, OTP) colour their own border plus `ring-2` with no
    offset, and an invalid field focuses in `ring-destructive-foreground` (6.16 light,
    6.61 dark), not a faded destructive;
  - calendar days use `ring-2` without offset so a ring never covers its neighbour.
- Touch: kit controls reach 44px under `pointer-coarse` through an absolutely positioned
  `::after` hit area (`min-h-11 min-w-11`, pattern in `ui/button.tsx`), without changing
  the desktop layout. A tooltip opens on a tap when its trigger has no action of its own; a
  trigger that is a working button or link keeps its click and shows the hint on hover and
  focus only.
- Disabled: a base rule gives `:disabled`, `[data-disabled]` and `[aria-disabled="true"]`
  `cursor: not-allowed`. Kit items blocked by Base UI (menu, select, combobox, tabs,
  slider, number field, drawer checkbox and radio rows, calendar days) keep pointer events
  so that cursor shows. Sidebar items keep `aria-disabled:pointer-events-none`, since
  `aria-disabled` alone stops nothing.

## Layout

- `.page-shell`: one gutter (`--page-gutter` 0.75rem, 1rem from `sm`) shared with the
  header, a per-page `--page-max` (default 1400px), padding-block 1.75rem 5rem.
- `.page-gutter` for pages that open with a full-bleed hero; `.page-bleed` to pull a
  sticky bar or a scroll rail out to the content edge.
- `--site-header-height`: 57px below `sm`, 65px from `sm`.
- `.scroll-shadows` (vertical) and `scroll-shadows-x` (horizontal): CSS-only edge shadows
  on a scroller, so a cut line reads as "more". `--scroll-shadow-bg` must be the surface
  under the scroller (defaults: popover for the vertical class, background for `-x`).
- Body reserves `--music-bar-reserve` while the music player bar is mounted.

## Components

`src/components/ui` is a shadcn-style kit on Base UI with `cva` and `tailwind-merge`.
Icons are lucide-react. Charts are recharts. Strings live in `*.messages.ts` next to the
component, never inline.

- Pressed looks, one per job. Multi-select filter chips are `FilterChip`; single-select
  segments and toolbar toggles are `Toggle` / `ToggleGroup`, pressed as an accent tint
  (`bg-primary/12`, `primary/16` in dark, a step more on hover) plus `inset-ring-ring`,
  with foreground text and no check, since the group already says one is chosen. A `ToggleGroup` drops the 44px coarse floor
  along its axis (`min-w-auto` horizontal, `min-h-auto` vertical) so neighbouring hit areas
  do not overlap.
  A disclosure that opens a panel (operators Filter, music Filters) stays a `Button` with
  `aria-expanded` and shows open through `data-pressed` with the same pressed classes; a
  Toggle would add `aria-pressed` on top of `aria-expanded`.
- `FilterChip` (`label`, `active`, `count?`, `tone?`, `toneInk?`): a kit Toggle, so it
  carries `aria-pressed` and the coarse hit area. Pressed shows a check mark, a `--tone`
  fill at 10% (15% on hover), a solid border and foreground text. The border is `toneInk`
  (an `-ink` token) when given, else the tone, else `--ring`; the old `--tone/50` measured
  2.28 light / 2.79 dark on the page. Every caller with a data `tone` passes its ink. The
  release-schedule kinds have no token, so `KIND_STYLE.ink` is a literal oklch of a Tailwind
  step that clears 3:1 on both pages (sky-600, yellow-700, teal-600); pink-500 and
  violet-500 go without one, since they clear it as they are.
- Static data badges: `<Badge variant="tint">` with `--tone` set inline (primary by
  default): tone fill at 12%, tone border at 45%, foreground text.
- Tabs: a horizontal kit `TabsList` scrolls with a hidden scrollbar, adds
  `scroll-shadows-x` only while it overflows (`data-overflow`), and scrolls the active tab
  into view. `useTabStripScroll` does the same for strips that are not a kit `TabsList`.
- Error states read `isError` before the empty and skeleton branches and use kit
  `LoadFailed` (`ui/load-failed.tsx`): the caller's "Couldn't load <thing>." line, the
  query's `refetch` behind a "Try again" button, never the server's text. `layout="inline"`
  is one row with no chrome, for a failure inside a card, dialog or section;
  `layout="block"` (default) is a centred bordered panel standing in for a whole tab or area.

## Do not touch from a design pass

OG card renderers (the operator card is pixel-locked), dynchar / pixi / chibi / spine
viewers, story reader plates, PDF / EPUB export renderers, `src/types/generated/**`,
`routeTree.gen.ts`.
