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
| subtle-foreground | `oklch(0.52 0.01 285)` | `oklch(0.62 0.01 285)` |
| primary / ring | `oklch(0.58 0.22 25)` red | `oklch(0.75 0.15 25)` salmon |
| primary-ink (text-primary) | = primary, 4.59 page / 4.79 card | = primary, 8.49 page / 8.07 card |
| primary-hover | primary 88% + black, label 6.08 | primary 88% + white, label 9.46 |

The one accent is the red primary (also `chart-1`, `enemy-boss`, `sidebar-primary`).
`--primary` is the fill. `text-primary` and `decoration-primary` read `--primary-ink`, and
`ring-primary` / `outline-primary` read `--ring` (namespace keys in `@theme inline`), so a
picked accent keeps its fill while text and ring get variants that clear 4.5 and 3 on the
page. `lib/theme/accent-kit.ts` is the one implementation (the pre-paint script in
`__root.tsx` is its `toString()`): a custom hex stays the fill and its label is black or
white by the higher ratio; a preset hue solves light L per hue (0.54 to 0.58, chroma clamped
to sRGB) to clear 4.5 on page and card and under its label, 3 on muted. Hover and pressed
fills use `bg-primary-hover`, never `bg-primary/90` (which measured 4.16 page / 4.14 card).
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
In code, `rarityInk(n)` (`lib/utils`) returns `var(--rarity-n-ink)` for text and
glyphs; `rarityStarColor` / `RARITY_HEX(_MUTED)` stay for fills, borders and rings.

The other data colors follow the same rule, with an `-ink` twin for text and glyphs
(same hue, darkened in light until it clears 4.6 on background and card and 4.5 on a 15%
tint of its own plain hue; dark uses the plain hue, which already measures 5.54 to
10.68 on the dark card and 4.63 to 8.04 on its tint). Ratios are light background /
card / own 15% tint, then dark card / tint:

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

All map to `text-*-ink` utilities. Code reaches them through `LEVEL_TOKENS[..].ink` /
`DAMAGE_TOKENS[..].ink` (enemies), `ENEMY_LEVEL_INK` (stage detail), `STAGE_GROUP_INK`
(IS reuses `--dmg-magic-ink`, Paradox `--rarity-4-ink`, Annihilation
`--destructive-foreground`) and `BANNER_STATUS_INK` (gacha). Like the rarity inks, none
is text-safe on the muted surface at every hue (4.07 to 4.62). A tone chip that sits on
a busy tint (a selected filter pill, a status pill on a tinted row) keeps the tone on its
border and fill and takes `foreground` text; a chip on an art scrim takes white. Free-form
user colours (tier-list flairs) never become text. A label painted ON a user colour (a tier
row's pill) takes black or white from `readableTextColor`, whichever has the higher WCAG
ratio against it.

Game rich-text spans in operator descriptions (`<@ba.kw>`, `<$ba.magic>`, `vup`, `vdown`,
`rem`) take `--game-kw-ink` / `--game-vup-ink` / `--game-vdown-ink` / `--game-rem-ink`
through `inkForTag` (operators/detail/impl/description.ts). Light: `oklch(0.5 0.12 194.3)`
5.14 / 5.37 / 4.50, `oklch(0.52 0.12 261.3)` 5.34 / 5.57 / 4.67, `oklch(0.53 0.12 24.7)`
5.37 / 5.61 / 4.70, `oklch(0.53 0.12 70.8)` 5.18 / 5.41 / 4.53 (background / card /
muted; the game's #27e8e7 measured 1.5 there). Dark keeps the game's own colours, 5.83 to
13.21 on background, card and muted. The story reader keeps `colorForTag` unchanged.

Legacy tokens left from the TanStack starter (`--sea-ink-soft`, `--lagoon`,
`--lagoon-deep`, `--palm`, `--surface`, `--bg-base`) survive only for the randomizer and
the leaderboard constants; the rest were deleted on 2026-11-03 with zero users.
`--glow-primary(-intense)` is used by the changelog and profile tabs CSS modules and
`__root.tsx`.

Text has three steps: foreground, muted-foreground for secondary text, and
`text-subtle-foreground` for the quietest meta (counts, ids, timestamps, kickers,
separators, empty-slot placeholders, placeholder text). Subtle measures 5.29 / 5.52 /
4.63 on background / card / muted in light and 5.51 / 5.24 / 4.75 in dark. Text never
takes an alpha step (`text-muted-foreground/70`, `/50`): over the page those measured
2.32 to 4.47, and 128 sites failed AA before the token replaced them. `/90` still clears
4.5 on every surface and is tolerated; anything quieter uses subtle. On art or a
translucent scrim, measure against the real backdrop instead.

Status text uses the `*-foreground` tokens (`text-success-foreground` 5.14,
`text-warning-foreground` 4.82, `text-info-foreground` 6.55, `text-destructive-foreground`
6.16 on the light page; `text-destructive-foreground` is red-400 in dark, 6.61 on the dark
card, where `text-destructive` measured 1.80 on its own tint), never a raw `-300` /
`-400` / `-500` utility, which measures 1.52 to 3.82 there.

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
  Kept loops: the stage map's tile effects (`stages/detail/map/impl/effects.css`) copy the game's animated tiles, so the motion is data that tells tile kinds apart; reduced motion still stops them.
- Focus: one base `:focus-visible` outline, 2px solid `--ring` at a 2px offset (4.59 light,
  8.06 dark); kit controls that draw their own ring set `outline-none`. That ring is
  solid `ring-ring` at `ring-2`, never an alpha step: `ring-ring/24` measured 1.46 and
  `/60` 2.70, under the 3:1 a focus mark needs. Kit fields dropped from a 3px halo to the
  same 2px solid ring. An invalid field focuses in `ring-destructive-foreground` (6.16
  light, 6.61 dark), not `ring-destructive/16`, which faded the focus mark exactly where
  the user has to fix something.
  Kit standalone controls (button, toggle, badge, switch, checkbox, radio, slider, menu
  switch, scroll-area, accordion trigger, tab panel) draw `ring-2 ring-ring ring-offset-2
  ring-offset-background`, the same geometry as the base outline. Exceptions, each for a
  reason: full-width list rows and anything inside a scroller (drawer rows, sidebar rows,
  tab triggers, which sit in horizontally scrolling strips) use `inset-ring-2` so the ring
  is not clipped; fields (input, select, textarea, OTP) colour their own border plus `ring-2`
  with no offset; calendar days use `ring-2` without offset so a ring never covers its
  neighbour.
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

- Toggle chips in a filter row: `FilterChip` (`label`, `active`, `count?`, `tone?`). It is
  a kit Toggle, so it carries `aria-pressed` and the coarse 44px hit area; pressed shows a
  check mark, a tone tint and border, and foreground text. Data-coloured text on its own
  tint measured 1.66 to 3.96, which is why the tone never colours the label. The pressed
  border is solid: `toneInk` (an `-ink` token) when given, else the tone, else `--ring`;
  the old `--tone/50` measured 2.28 light / 2.79 dark on the page. Every caller with a data
  `tone` passes its ink; the release-schedule kinds have no token, so `KIND_STYLE.ink` is a
  literal Tailwind step that clears 3:1 on both pages (sky-600, yellow-700, teal-600), and
  pink-500 / violet-500 go without one, since they clear it as they are.
- Pressed looks: multi-select filter chips are `FilterChip` (tone tint, solid edge, check
  mark); single-select segments and toolbar toggles are `Toggle` / `ToggleGroup` (grey fill
  plus `inset-ring-foreground/40`, no check, since the group already says one is chosen).
  A horizontal `ToggleGroup` drops the 44px coarse width floor on its items
  (`min-w-auto`) so neighbouring hit areas do not overlap; heights still reach 44.
- Static data badges: `<Badge variant="tint">` with `--tone` set inline (primary by
  default): tone fill at 12%, tone border, foreground text.
- Error states read `isError` before the empty and skeleton branches and offer a retry;
  reuse `LoadError`, `LoadFailed` or `PanelRetry` rather than a new shape.

## Do not touch from a design pass

OG card renderers (the operator card is pixel-locked), dynchar / pixi / chibi / spine
viewers, story reader plates, PDF / EPUB export renderers, `src/types/generated/**`,
`routeTree.gen.ts`.
