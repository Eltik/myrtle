# myrtle.moe Design System

> Design system extracted from **[Eltik/myrtle @ v3](https://github.com/Eltik/myrtle)** — an Arknights
> game companion platform (operator database, profile sync, tier lists, DPS charts, recruitment
> calculator, asset browser).

Named after the Arknights operator **Myrtle**. The site calls itself _"the most comprehensive,
accurate, and user-friendly Arknights resource available."_ Stack: **TanStack Start** (Next/Vite-like
SSR) + **Tailwind CSS v4** + **@base-ui/react** primitives, with a bespoke token layer on top.

## Products represented

One product, one surface:

- **myrtle.moe web app** (`frontend/`) — SSR marketing/home + authed dashboards (operators,
  tier-lists, DPS charts, recruitment, randomizer, settings). No native/mobile app in the repo.

Rust backend (`backend/`) exists but has no UI — game-data API only.

## Sources

- GitHub: `Eltik/myrtle` branch `v3` (default is `main`; v3 is the active rewrite)
  - `frontend/src/styles.css` — all design tokens live here
  - `frontend/src/components/ui/*.tsx` — base-ui-react + Tailwind wrappers
  - `frontend/src/components/header/*.tsx` — app shell
  - `frontend/src/routes/*.tsx` — route layouts
- No Figma, no Slack-dropped decks.

## Index

```
README.md                ← this file
SKILL.md                 ← Agent-Skill front matter so this folder is portable
colors_and_type.css      ← all color + type tokens, light + dark
fonts/                   ← (Google Fonts CDN used; see note below)
assets/                  ← logo mark, favicons
preview/                 ← design-system preview cards (Type, Colors, Spacing, Components, Brand)
ui_kits/
  myrtle-web/            ← hi-fi recreation of the web app shell
    index.html
    README.md
    *.jsx                ← modular components
```

## Fonts

- **Inter** — self-hosted variable TTF at `fonts/Inter-VariableFont_opsz_wght.ttf`. Used for
  all sans text.
- **Geist Mono** — self-hosted variable TTF at `fonts/GeistMono-VariableFont_wght.ttf`. Used
  for `code`, `kbd`, `pre`.

(No display serif — v1 myrtle.moe is Inter + Geist Mono only.)

---

## Content fundamentals

See sections below — populated as the system is built out.

- [Content fundamentals](#content-fundamentals-1)
- [Visual foundations](#visual-foundations)
- [Iconography](#iconography)

## Content fundamentals

**Voice.** Technical-precise, understated, player-to-player. The site never talks like a
marketing page; it talks like a well-written dev README. Present tense, active voice, no
exclamation marks, no em-dashes between clauses-for-drama. Copy assumes the reader is already
an Arknights player and doesn't over-explain game mechanics.

**Person.** Mostly 3rd person descriptive ("Complete information on all 400+ Arknights
operators"). 2nd-person "you" only appears in direct instruction ("Link your Arknights
account to view your roster"). Never 1st person plural ("we") outside the footer.

**Casing.** **Sentence case everywhere** — UI labels, buttons, nav, page titles. Title Case
appears only in proper nouns ("Recruitment Calculator", "DPS Charts" — where the feature
has a formal name) and on the **kicker** micro-labels which are `UPPERCASE TRACKED +0.16em`.

**Tone flags.**
- No emoji. Zero. Not in copy, not in UI, not in commit messages.
- No unicode decoration (✨ ▲ • etc). Separators are literal `·` or a Separator component.
- Abbreviations stay uppercase: DPS, CN, EN, UID, JWT.
- Numbers: digits always, no "four hundred" prose. "400+ operators" is the house style.

**Examples — directly from the repo:**
- Hero h1 (about page): _"A small starter with room to grow."_ — a full stop, no exclamation.
- Feature bullet: _"Operator Database: Complete information on all 400+ Arknights operators
  including stats, skills, talents, modules, skins, voice lines, and lore."_
- Disclaimer footnote: _"Note: This project is not affiliated with Hypergryph, Yostar, or any
  official Arknights entities."_

**Kicker pattern.** Above hero titles, a tiny uppercase word sets context: `ABOUT`, `OPERATORS`,
`TIER LISTS`. The kicker is `0.69rem / 700 / letter-spacing: 0.16em`, colored in `--primary`
(the coral).

## Visual foundations

**Palette strategy.** Two layers:
1. A **brand layer** of warm-sea colors — `sea-ink` (dark teal), `lagoon` (aqua),
   `palm` (green), `sand`, `foam`. These ship as raw hex vars.
2. A **semantic layer** on top (shadcn-style) in `oklch()` — `--primary`, `--muted`,
   `--destructive`, etc. `--primary` is a **warm coral** `oklch(0.58 0.22 25)`, _not_ teal —
   teal is the accent/atmosphere, coral is the action color. This is the single most important
   thing to preserve when building myrtle UI.

**Background.** `--bg-base` is a very pale mint `#e7f3ec` in light, nearly-black teal
`#0a1418` in dark. The hero at `/` uses two radial washes: `--hero-a` (lagoon 36%) + `--hero-b`
(palm 20%) layered over the base.

**Type.** Inter Variable for sans (100–900 axis). Geist Mono Variable for code/kbd.
No display serif — `.display-title` is just Inter 700 with tight tracking.

**Spacing.** Tailwind default scale. Page wrap is `width: min(1080px, calc(100% - 2rem))`,
centered. Headers are `h-14 sm:h-16`, buttons are `h-9 sm:h-8` at default size (condensed
on desktop, taller on mobile — **mobile-first inverted**).

**Radius.** `--radius: 0.625rem` (10px) is the atom. Derived: sm ~6px, md ~8px, lg 10px,
xl 14px, 2xl 18px, 3xl 22px, 4xl 26px. **Cards use 2xl (18px)**. Buttons use lg (10px).
Badges use sm (~4–6px). Nothing is a perfect circle except avatars and the logo bezel.

**Shadows.** Two systems running in parallel:
1. **`shadow-xs/5`** — a 5%-opacity hairline under inputs/buttons/cards. Paired with a
   **`before::shadow-[0_1px_black/4%]`** inset glint to emulate a physical edge.
2. **`island-shell`** — the big layered card shadow:
   `0 22px 44px foreground/10 + 0 6px 18px foreground/8`, combined with `backdrop-filter: blur(4px)`.
   Used on the hero island and large hero containers.

**Inner glint.** In dark mode, the inset glint flips to `0 -1px white/6%` — a top-edge
highlight. This is applied via Tailwind v4 `before:` pseudos, not box-shadow directly.

**Hover states.** Buttons darken (`hover:bg-primary/90`, the `/90` is the magic number — 10%
darker). Outline/ghost buttons get `hover:bg-accent` (near-white in light, near-black in dark).
Nav links animate a 2px underline from left via `transform: scaleX()` over 170ms ease. No
color-shifts on nav hover — just the underline reveals.

**Press states.** `data-pressed:bg-primary/90` (same as hover, matching Base UI conventions)
_plus_ the inset-glint flips from `0 1px white/16%` to `0 1px black/8%` — the button looks
recessed.

**Borders.** `--border` is `oklch(0.88 0.005 285)` in light — ~12% of a neutral cool-gray.
Everything with a border uses `1px solid var(--border)`. Inputs, cards, buttons, separators.

**Transparency + blur.** The header bar is `bg-background/95 backdrop-blur-md supports-backdrop-filter:bg-background/60`.
Other sticky surfaces follow this pattern. The brand `--surface` / `--surface-strong` vars are
RGBA for use on translucent panels overlaying the hero gradient.

**Imagery.** Arknights splash art is warm-saturated anime illustration; the site counterbalances
with cool mint/teal chrome so the art pops. Full-bleed imagery is avoided on core chrome —
images live inside cards with `rounded-2xl` bezels.

**Animation.** One named keyframe: **`rise-in`** — `700ms cubic-bezier(0.16, 1, 0.3, 1)` with
12px translate and opacity fade. Page sections use this on mount. Transitions are all
`180ms ease` on background/color/border/transform. No bounces, no springs, no parallax.

**Layout rules.** `.page-wrap` caps at 1080px. Sidebar (when present) is 16rem. Header
sticky with `z-50`. Cards are always inside `.page-wrap`, never full-bleed.

## Iconography

**Source.** [`lucide-react`](https://lucide.dev) — imported as individual components, e.g.
`import { ChevronDown } from "lucide-react"`. Default size is `size-4` (16px) on desktop
and `size-4.5` (18px) on mobile via the button variant CVA. Stroke width 2.

**Weight / fill.** All icons are **stroke**, weight 2, `stroke-linecap: round`,
`stroke-linejoin: round`. Never filled. Opacity defaults to 80% so icons recede next to text.

**Logo mark.** One custom inline SVG — three stacked hexagon layers (see
`assets/logo-mark.svg`). Rendered inside a `bg-linear-to-br from-primary to-lagoon`
rounded-full bezel, 28px outer, 14px icon.

**Emoji.** Never used. If something would be an emoji, it's a lucide icon instead.

**Unicode as icons.** Never. `·` appears as a text separator only.

**Substitution note.** We use the [Lucide CDN](https://unpkg.com/lucide-static) for preview
cards and HTML mockups. In React, import from `lucide-react` to match the original.

---

See `preview/` for visual cards of each part of the system. See `ui_kits/myrtle-web/` for a
working recreation of the app shell.
