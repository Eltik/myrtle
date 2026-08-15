# design-sync notes — myrtle frontend

Repo-specific gotchas for future syncs. Read this before touching `.design-sync/config.json`.

## The shape of this sync

- **This is an application, not a component library.** There is no `dist/` library
  entry, no barrel, no `exports` field — `dist/` holds a stale Vite SSR app build
  and the live build target is `.output/` (Nitro). The converter therefore runs in
  synth-entry mode over `src/components/`.
- `srcDir` is `src/components` (not `src`): pointing it at `src` would pull routes,
  integrations and the OG pipeline into the bundle.
- Scope decision (2026-08-14, user): **everything** under `src/components` — ui
  primitives, presentational feature components, router-coupled components, and
  data-coupled containers. 329 files → 697 exports → 692 component cards.

## Why `overrides/source-kit.mjs` is forked

Declared in `cfg.libOverrides`. Two reasons, both structural to this repo:

1. The bundled adapter synthesizes its entry as `export * from` every source file.
   22 names are declared in 2–4 files each (`Kicker`, `Hero`, `Pagination`,
   `StatCard`, `Toolbar`, `CompactCard`, …). ESM makes an ambiguous star-export
   name unresolvable, so all of them would silently vanish from `window.MyrtleUI`.
   The fork emits explicit named re-exports and disambiguates with the feature dir
   (`HomeHero`, `LeaderboardHero`, `AdminKicker`, …); **`ui/` always keeps the bare
   name** because it is the design-system layer. 39 exports get renamed — the build
   log prints the full old→new list every run.
2. The bundled grouping heuristic treats `ui` as a generic container, so every
   primitive would land in group `general`. The fork groups by the top-level dir
   under `src/components` and maps `ui` → `primitives`.

If you ever re-scope, the fork is the only place the file list is decided.

## The stub layer (`.design-sync/stubs/`, wired via `tsconfig.ds.json` paths)

An app's component tree reaches server-only code through shared helpers. Each stub
below fixes a build or load failure that blocks the *entire* bundle, not one card:

| Stub | Fixes |
|---|---|
| `env.ts` | `src/lib/utils.ts` (home of `cn()`) imports `#/env`, so t3-env's `process.env` access made **all 692 cards** throw `ReferenceError: process is not defined`. |
| `tanstack-react-start.ts` + `-server.ts` | `#/lib/api/*` define fetchers with `createServerFn`; the real package drags `@tanstack/start-server-core` in, which needs `node:stream`, `node:crypto` and `#tanstack-start-*` virtual entries. |
| `route.ts` | 5 containers import `#/routes/*` for typed search params; a real route module pulls in `routeTree.gen.ts` and the satori/resvg OG pipeline. |
| `node-crypto.ts` | `src/lib/api/tier-lists.ts` imports `randomBytes` for a share token. |

`stubs/env.ts` sets `VITE_BACKEND_URL` to `https://api.myrtle.moe` — the public host
from the repo's own `.env.example` — so preview cards resolve real operator/item/
enemy artwork instead of broken images. Nothing secret goes in that file; it ships
inside the uploaded bundle.

## `tsconfig.ds.json` traps

- **Use `//` line comments, never a `"//"` JSON key.** The converter's paths plugin
  strips comments with a regex that mangles `"//": "..."` into invalid JSON, then
  swallows the parse error and returns *no plugin at all*. Symptom: stubs appear to
  be ignored and the build fails with the exact errors they were meant to fix.
- **Key order is load-bearing.** The plugin takes the first matching rule, so every
  exact entry must sit above the `#/*` wildcard.
- `#/lib/<dir>` directory imports need explicit entries (`#/lib/export`,
  `#/lib/markdown`, `#/lib/og`, `#/lib/api/admin`). The plugin's extension probe
  tests `existsSync(stem)` before `stem/index.ts`, and a directory passes — esbuild
  then reports `Cannot read file …: is a directory`.
- `#/*` also resolves natively via package.json's `imports` field, so a broken
  paths plugin does **not** produce `#/…` resolution errors. Don't use that as your
  signal that path mapping is working.

## CSS

`src/styles.css` is Tailwind v4 *source* (`@import "tailwindcss"`, `@theme`,
`@plugin`), not shippable CSS. `.design-sync/build-css.mjs` compiles it with the
Tailwind CLI into `.design-sync/.cache/tailwind.css` (`cfg.cssEntry`) and repoints
the `@fontsource` `url(./files/*.woff2)` references at `node_modules` so the
converter's `@font-face` scraper can copy them into `fonts/`. **Run
`node .design-sync/build-css.mjs` before every converter run** (it is `cfg.buildCmd`).

The compiled sheet contains only utilities used by `src/**`. Fraunces
(`--font-display`) loads from a remote Google Fonts `@import` → `[FONT_REMOTE]` is
expected and needs no action.

## Providers

`.design-sync/preview-provider.tsx` exports `DesignPreviewProvider` (`cfg.provider`,
added to the bundle via `cfg.extraEntries`). It supplies router + query + toast +
command context. The router uses `RouterContextProvider` — the only way to get
`<Link>` working without mounting a route tree.

## Preview authoring — what the waves learned

### Tailwind: an uncompiled class fails silently

The single biggest trap. `build-css.mjs` compiles `.design-sync/tailwind-entry.css`,
which now scans `src/`, `.design-sync/previews/`, **and** a generated safelist
(`.cache/safelist.txt`, emitted by `build-css.mjs`) covering common layout,
typography and token utilities with their `sm/md/lg/xl:`, `dark:`, negative and
`/opacity` forms. Anything outside that set produces no error and no `⚠` cell —
the element simply renders unstyled, so a wrong card grades `good`. Arbitrary
variant selectors (`[&>button]:text-primary`) are essentially never compiled.
Check with `grep -c 'the-class' .design-sync/.cache/tailwind.css` (variants are
backslash-escaped in the output: `md\:grid-cols-3`).

The safelist exists mostly for **designs**, not previews: designs built in
claude.ai/design are not run through Tailwind, they consume this stylesheet as-is.
Widening the safelist is how you widen what the design agent can express.

### Base UI behaviours that cost a fix round

- `Select`/`Combobox` value components: a children **render function** shadows
  `placeholder` (blank trigger). `AutocompleteValue` is the exact mirror — its
  `placeholder` never fires because the input string counts as a selected value,
  so there you *need* the render fn. Check per component.
- Root-only props are silently dropped on children: `defaultValue`,
  `defaultInputValue`, `grid`, `multiple`, `disabled` belong on the root.
- `defaultOpen` is ignored when a `handle` is attached — use `open`.
- `AutocompleteEmpty` / `CommandEmpty` only render when the root has `items`
  (`items={[]}` for the empty story).
- An open modal focuses its first tabbable control, and the DS focus ring is brand
  red — every open-dialog card ships one red-ringed control. Disabling every
  action moves that ring around the whole popup, which reads as a validation
  error. `initialFocus={false}` is the escape hatch.
- `*Primitive` re-exports (`DialogPrimitive`, `AlertDialogPrimitive`,
  `CommandDialogPrimitive`) are **not** in the bundle's export set even though the
  source re-exports them. Portal/Backdrop/Viewport subparts therefore share their
  parent's composition — which is the honest preview anyway, since `DialogPopup`
  already mounts all three.

### 🚨 Never edit config or `overrides/` while authoring agents are running

`preview-rebuild.mjs` refuses to run with `[CONFIG_STALE]` when
`.design-sync/config.json` or any `.design-sync/overrides/*.mjs` is newer than the
stamp in `ds-bundle/.stories-map.json`. `configSlicesFor()` hashes every override
file into a *global* slice, so one fork edit invalidates every component at once.
There is no `--force` or env bypass — the guard exits before any lib loads, and
only `package-build.mjs` re-stamps.

Applying mid-wave `CONFIG:` lines therefore **blocks every agent in that wave**.
Batch config and fork edits to wave boundaries: collect the `CONFIG:` lines, apply
them all at once, run one full `package-build.mjs`, and only then start the next
wave. (`package-capture.mjs` does not carry the guard — only the rebuild does.)

### Capture harness

- `.ds-single` sets `transform: translateZ(0)`, so `position: fixed` resolves
  against the story root, not the viewport. Open-overlay stories need an explicit
  stage height (`min-h-[520px]`) or the backdrop collapses to a strip.
- Capture is a 900×700 viewport shot, so a portalled open dialog lands centred and
  in frame — dialogs need **no** `cardMode` override. What *does* need
  `cardMode: single` is a popup that would paint over its neighbours in the
  multi-cell grid card (the Combobox and CommandDialog families — see
  `cfg.overrides`).
- ⚠️ **`package-capture.mjs` can hang forever.** Its `settle()` awaits
  `img.decode()` on every image with no timeout, and `OperatorAvatar` renders
  `loading="lazy"`; an avatar scrolled out of view never loads, so decode never
  resolves. Symptom: capture wedges with no output. Avoid long scrolling lists of
  avatars in a story; if a capture hangs, that is the first thing to check.
- Concurrency costs more than it looks: each agent spawns its own ~1.3 GB esbuild
  service, and a 19-component `preview-rebuild` took 8–12 minutes with 3–7 agents
  running. Six concurrent agents is about the practical ceiling on this machine.

### A real app bug this surfaced (not a sync issue)

`CardFrameFooter` in `src/components/ui/card.tsx` lacks `relative`, while
`CardFrameHeader` has it. `CardFrame` paints an `absolute inset-0
before:bg-muted/72` overlay, and positioned descendants paint above
non-positioned in-flow children — so plain footer content gets a 72% muted wash
and reads as disabled. Previews work around it with `className="relative"`.
Worth fixing at source.

## The stub layer is load-bearing — four bugs it fixes are invisible

Every one of these blanked cards **with `package-capture` reporting zero errors**.
The screenshot was the only signal. If a card is blank and capture is clean, serve
`ds-bundle/` and log `pageerror` (playwright resolves from `.ds-sync/node_modules`)
— it takes ~60 seconds and names the throw.

| Symptom | Cause | Fix |
|---|---|---|
| every card blank | `#/env` → t3-env reads `process.env` | `stubs/env.ts` |
| ~20 cards blank | `useAuth()` → `useRouteContext({from:"__root__"})` has no active match (21 call sites) | `stubs/use-auth.ts` |
| container blank | `Route.useSearch()` returned `{}`; containers destructure into state and throw on `.trim()`/`.length` | `stubs/route.ts` permissive defaults incl. `flair: []`, `tags: []` |
| bundle won't build | `createServerFn` drags in node builtins | `stubs/tanstack-react-start.ts` |

⚠️ **A stub's existence doesn't mean it's in the bundle.** `preview-rebuild.mjs`
does not rebuild `_ds_bundle.js` — only `package-build.mjs` does. Three separate
agents lost time diagnosing a hook that was already stubbed. Compare `stat` times
before diagnosing, or `grep -c` the stub's marker string in `ds-bundle/_ds_bundle.js`.

The server-fn stub's rejection message is deliberately **product-shaped**
("Couldn't reach the server…") because data-coupled containers render
`error.message` straight into their error branch — a harness-flavoured string leaks
into the cards.

## Seven components are un-previewable by construction

`EnemyDetail`, `OperatorDetail`, `StageDetail`, `StageList`, `TierListDetail`,
`TierListEditor`, `UserProfile`. They call `useParams({from:"/x/$id"})` /
`useSuspenseQuery` and need **active route matches**, which the preview provider
cannot supply: `RouterContextProvider` never runs matching, and a single memory
history location can only ever match one route, so no one location satisfies all of
them. A preview can't mount its own router either — `@tanstack/react-router` and
`@tanstack/react-query` bundle *separately* into each preview (only the package and
its exported components shim to `window.MyrtleUI`), so a provider written in a
preview has a different context identity than the bundled component reads.

Their authored previews are parked in `.design-sync/.cache/unpreviewable/` (correct
as written — they render the moment a router fix lands). They ship as **floor
cards**, which is the honest baseline: a blank card is worse than one that says so.
Fixing this needs the provider to expose real matches — see "Re-sync risks".

## Known render warns

These are triaged and benign — a re-sync should treat them as expected, and only
investigate a warn **not** on this list.

- `[RENDER_THIN] variants render identically` on ~70 overlay components — the
  Dialog, AlertDialog, Drawer, Sheet families plus every `*Dialog`/`*Sheet` feature
  component. Their content portals to `document.body`, so the measured story root
  is near-identical across stories even though the screenshots differ. Sheets were
  read individually and graded `good`.
- `[RENDER_THIN] rendered height is 0px` on `TierDetailsDialog` — same cause
  (portalled popup, zero-height root).
- `[RENDER_THIN]` on `ExportDialog`, `DetailOperatorNotes`, `OperatorFormSwitcher`.
- `[DTS_STYLE_SYSTEM] filtering @types/react props` — expected; the extractor is
  filtering React's own CSS-shorthand prop bag, not real component API.
- `[FONT_REMOTE]` — Fraunces (`--font-display`) loads from Google Fonts by
  `@import`. Expected; the other two families are self-hosted.

## Preview-authoring gotchas worth keeping

- **An uncompiled Tailwind class fails silently.** See the CSS section above. The
  presence check must be escaping-aware: Tailwind escapes `/`, `.`, `[`, `]`, `,`
  and `:` in selectors, so `grep -c 'bg-muted/10'` false-*negatives*, while
  `includes(".data-pressed\\:bg-primary")` false-*positives* against
  `data-pressed:bg-primary/90`. Confirmed uncompiled: `h-screen`, `max-h-screen`,
  `min-h-96`, `text-start`, `ps-*`/`pe-*` (all logical properties), `pl-64`,
  `h-56`, `p-7`, `w-[420px]`, `leading-[1.7]`, `max-w-[58ch]`.
- **The capture clock is frozen at 2024-05-15.** Date-derived fixtures must be
  sized against it or they render impossible figures ("1,876/1,583 days",
  "100% rate / 0 days missed"). Live API payloads are dated 2025–26, so anything
  date-derived needs re-stamping.
- **A wrong operator id renders a real portrait of the wrong operator** and returns
  HTTP 200. One agent shipped 9 before a diff against `/api/operators/index` caught
  them. The index spells `Młynar` and `Wiš'adel` with non-ASCII, and `Amiya` is
  three rows — look up by id, never by name.
- **`.ds-single` sets `transform: translateZ(0)`**, which makes the story root both
  the `position: fixed` container *and* the collision boundary Base UI measures
  against. Open-overlay stories need an explicit stage height (`min-h-[520px]`,
  `min-h-dvh`) and their width cap on the *content*, not the stage.
- **The auto-focus ring lies.** An open modal focuses its first tabbable control,
  and the ring is brand red — it lands on an empty required field and reads as a
  validation error. Fix: in the mount-click helper, fire two more `rAF`s after the
  click and `blur()` the active element.
- **Overlays with internal-only open state** (`FilterSheet`, `VersionsBar`,
  `SubscoreCard`) need a click-on-mount deferred **two `requestAnimationFrame`s** —
  firing from `useEffect` is silently dropped.
- **Capture can hang forever** on a *fixed, far-off-screen* subtree containing
  `loading="lazy"` images (`BasePanel`'s PNG-export poster at `-left-24999px`):
  `settle()` awaits `img.decode()` with no timeout. Lazy images merely below the
  fold are fine. Never capture a whole batch in one call — one hang stalls
  everything with no indication of the culprit.
- **`⚠` at the start of a story's text is a false capture error** — the harness
  treats it as a caught render error.
- Capture is `fullPage: false` at 900×700, so `lg:` never fires and `hidden lg:flex`
  components are invisible (pass `className="flex"`; tailwind-merge drops the
  `hidden`). Feature groups' constants (`PALETTE`, `TEXT_KICKER`, `CARD_PADDING`,
  `operatorRarityColor`) are **not** bundle exports — only `src/components/ui/*`
  lowercase exports are.
- PIXI/spine and recharts both **do** render in the capture harness.

## Source bugs found while authoring (not sync issues)

Worth fixing in `src/`; previews work around them.

1. `ui/card.tsx` — `CardFrameFooter` lacks `relative` (its header has it), so
   `CardFrame`'s `before:bg-muted/72` overlay washes footer content out to look
   disabled.
2. `tools/shared/AxisControls.tsx` — the sweep `NumberField`s are `w-full sm:w-24`;
   at ≥640px the steppers leave ~26px for a 58px value, so `To 3,000` renders as
   `To 3`. Reproduces in the real DPS/HPS pages. The card is pinned to a 600px
   viewport as a workaround.
3. `operators/list/impl/components/OperatorCardCompact.tsx` — `parseOperatorName`
   splits alter names ("Lappland the Decadenza") onto two lines that get centred
   into a 20px unclipped box, hiding the name behind the portrait.
4. `enemies/detail/impl/sections.tsx` — `descToHtml` matches `<@ba.*>` but the
   handbook ships `<$ba.*>` (~400 occurrences), which renders as literal markup.
5. `ui/progress.tsx` — `ProgressIndicator` emits no style for `value={null}`, so
   indeterminate progress renders an empty track.
6. `ProgressValue`/`MeterValue` ignore `max` and append `%` by default, so
   `value={3164} max={4812}` renders "3,164%".

## Re-sync risks

- **The seven un-previewable containers are the one open improvement.** If someone
  gives `preview-provider.tsx` a router with real matches — a route tree containing
  `/user/$id`, `/operators_/$id`, `/enemies_/$id`, `/stages_/$stageId`,
  `/tier-lists_/$id` plus `await router.load()`, or a `RouterProvider` whose root
  route renders the story — restore the previews from
  `.design-sync/.cache/unpreviewable/` and re-capture. They are written and correct.
  Note a single location matches only one route, so this needs per-story routing,
  not one global location.
- **Root-relative assets.** Components hardcode `/logo/*` and `/stat-icons/*`.
  Those are copied into the bundle by the close-out step and included in the upload
  plan — if a future run rebuilds without that copy, the brand mark and combat-stat
  rows show broken-image glyphs. The logo is downscaled from the 2.8MB source.
- **Fixtures are pinned to live API data** from `api.myrtle.moe` (operators,
  enemies, stages, skins). If a payload's shape changes, a preview can render
  wrong-but-plausible content rather than failing. The frozen 2024-05-15 clock
  means date-derived values need re-checking whenever fixtures are refreshed.
- **Grades are only as good as the sheets.** Any preview written but never captured
  is a draft — three separate agents died mid-batch and left files that looked
  finished; re-reading the sheets found blank cells, nine wrong operator portraits,
  and impossible arithmetic. Never carry an ungraded preview forward as done.

- **`stubs/` shadow real modules.** If `src/lib/api/*`, `#/env`, or the route
  modules change their exported surface, the stubs go stale silently — the build
  still succeeds, the affected components just misbehave. Re-check the stub exports
  against the real modules when API modules move.
- **The fork's rename list is API.** `AdminKicker`, `HomeHero`, etc. are the names
  the design agent codes against. Adding a component that collides with an existing
  name can *reshuffle* which file wins a bare name — watch the build log's
  `disambiguated N:` line for unexpected changes, and diff it against this list.
- `cfg.cssEntry` points into the gitignored `.cache/`. A fresh clone must run
  `build-css.mjs` first or the build ships no CSS.
- The Tailwind CLI version in `build-css.mjs` is pinned (`4.2.2`) and fetched via
  `bunx`; it should track the repo's `tailwindcss` dependency.
