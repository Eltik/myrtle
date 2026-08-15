# Building with Myrtle UI

Myrtle is an Arknights companion site. Its components are **Base UI**
(`@base-ui/react`) primitives wrapped with Tailwind v4, plus feature components
built from them. Everything is on `window.MyrtleUI`.

## Wrap the tree in `DesignPreviewProvider`

Components read four contexts — router (anything with a link), query (anything that
fetches), toasts, and the ⌘K command palette. Without the wrapper those components
render **blank, with no error in the console**.

```jsx
const { DesignPreviewProvider, Button, Card, CardHeader, CardTitle, CardPanel } = window.MyrtleUI;

<DesignPreviewProvider>
  <Card className="max-w-sm">
    <CardHeader>
      <CardTitle>Chapter 8 — Roaring Flare</CardTitle>
    </CardHeader>
    <CardPanel className="text-muted-foreground text-sm">
      Recommended average level: <span className="font-medium text-foreground">E2 40</span>
    </CardPanel>
  </Card>
</DesignPreviewProvider>
```

Theming needs no provider: light is the `:root` default and dark is a `.dark` class
on `<html>`.

## Styling idiom: Tailwind utilities over semantic tokens

Style your own layout with Tailwind utility classes. **Always prefer the semantic
token name over a raw colour** — `bg-card`, not `bg-white`. The exact families that
exist:

| Family | Values |
|---|---|
| `bg-` | `background card popover muted accent primary secondary destructive sidebar foreground` |
| `text-` | `foreground muted-foreground card-foreground popover-foreground primary primary-foreground secondary-foreground accent-foreground destructive success warning info` |
| `border-` | `border input primary destructive transparent` |
| `font-` | `sans heading mono display` + `normal medium semibold bold` |
| `rounded-` | `sm md lg xl 2xl 3xl full none` |

Opacity modifiers work on the colour families (`bg-primary/10`), as do `sm:`/`md:`/
`lg:` and `dark:`.

**Domain tokens are CSS variables only — there is no utility class for them.** Use
them inline: `style={{ backgroundColor: "var(--rarity-6)" }}`. Available:
`--rarity-1…6`, `--enemy-normal|elite|boss`, `--dmg-physic|magic|heal|none`,
`--chart-1…5`, `--glow-primary`, plus the nautical brand palette (`--sea-ink`,
`--lagoon`, `--sand`, `--foam`, `--surface`, `--line`).

The stylesheet ships a **fixed** set of utilities — it is not compiled per design.
The families above plus common layout/spacing/typography are guaranteed; an exotic
arbitrary value (`w-[820px]`, `leading-[1.7]`) will silently do nothing. Prefer the
standard scale, or an inline `style`.

## Helpers, not just components

`cn(...)` merges Tailwind classes (clsx + tailwind-merge) — use it for conditional
classes. `buttonVariants`, `badgeVariants`, `toggleVariants`, `groupVariants` and
`selectTriggerVariants` give a non-button element the same look. `toastManager` and
`anchoredToastManager` raise toasts imperatively.

## Compound parts are flat exports

Base UI, not Radix: `SelectPopup`, `DialogPopup`, `CardPanel` — never
`Select.Content`. A subpart only renders inside its family root (`MenuGroupLabel`
throws outside `MenuGroup`).

**39 feature components are renamed to break collisions with the primitives**, and
the renamed name is the API: `HomeHero`, `TierListsHero`, `LeaderboardHero`,
`ProfileHero`, `AdminKicker`, `StatsKicker`, `DetailKicker`, `ListPagination`,
`AdminStatTile`, `ItemsCompactCard`, `RosterCompactCard`, `MapTile`, `StatsTile`…
`ui/` always keeps the bare name (`Kicker`, `Pagination`, `Toolbar` are the
primitives). Check the component's own folder name when in doubt.

## Where the truth is

Read `_ds/<folder>/styles.css` and its imports for the token layer, and each
component's `<Name>.d.ts` (real props) and `<Name>.prompt.md` before styling around
it. The preview card for a component is a working example of its intended use.
