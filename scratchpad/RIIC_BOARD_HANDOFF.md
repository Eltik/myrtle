# RIIC Base Board — Handoff

**Goal:** Make the Optimizer tab's RIIC board visually match the Base Builder at
`https://tools.adachurch.com/#/base`.

**Current state:** Broken. Tiles render as one horizontal strip, full-colour room
artwork is stretched across each cell, and facility names overflow their tiles.

---

## Root causes (all verified)

### 1. Tiles carry no coordinates — this is the blocker

`frontend/src/lib/base/layout.ts` claims:

> "`slot_id` is an opaque key (`slot_1`) in both `building_data` and the player's
> `roomSlots`, so the board's real geometry has to come from somewhere else"

**This premise is false.** The geometry is in `building_data.json` (see next
section). Because `ITile` has no row/col, `Board.tsx` lets tiles auto-flow into
the grid, which produces the single-row strip.

### 2. Most emitted class names have no CSS

`Board.css` styles `.riic-tile` plus state modifiers, and nothing else.
`RoomTile.tsx` emits all of these with **zero** matching rules:

- `.riic-tile-room-clip`
- `.riic-tile-room-body`
- `.riic-tile-room-accent`
- `.riic-tile-info`
- `.riic-tile-name-row`
- `.riic-tile-name`
- `.riic-tile-level-pips`
- `.riic-tile-level-pip`

`.riic-tile-name` is therefore unstyled text inside a `display:flex` +
`justify-content:center` + `overflow:hidden` tile, so long names wrap mid-word
and clip. Pips render as zero-size spans.

Also: `.riic-tile` sets `align-items:center; justify-content:center`. Once the
children are absolutely positioned these do nothing useful — remove them.

### 3. Wrong image assets

`riicRoomIcon()` in
`frontend/src/components/operators/detail/impl/assets.ts` points at
`textures/arts/building/architect/room_icon_sprite_hub/{Control,Trading,...}.png`.

Those are **328x328 full-colour room artwork**, not glyphs. They are then painted
as `background-image` at `background-size: 100% 100%`, stretching illustrations
across each cell.

Adachurch instead uses a flat silhouette as `mask-image` on a dedicated
`.riic-tile-room-icon` element, with colour supplied by CSS underneath. The
closest monochrome assets in our extraction are `icon_title_*.png` (40x44) in the
same directory — small enough that hand-tracing nine SVGs into
`frontend/public/riic/` is the better option.

### 4. Structure tiles are mis-classed

`RiicTile.tsx` renders elevators and corridors with `.riic-tile-empty`
(`opacity: 0.6`). Adachurch uses dedicated `.riic-tile-elevator` /
`.riic-tile-path` — the lift shaft is a visible architectural element, not a
dimmed blank.

`RoomTile.tsx` also hardcodes `riic-tile-flexible` on every room. The control
centre is `riic-tile-fixed` on adachurch and has a completely different subtree
(see Reference DOM).

---

## The geometry data

Verified in `assets/output/en/gamedata/excel/building_data.json`.

Note the unpacker PascalCases keys — it is `Rooms` and `Layouts`, not `rooms` /
`layouts`. Both are serialised as `[{key, value}, ...]` pair-lists, **not** maps.

`Layouts[0].value` has `Id: "v0"`, `Slots` (51 entries), `Storeys` (5), and
`CleanCosts`. Each slot:

```json
{
  "Id": "slot_1",
  "Category": "ELEVATOR",
  "StoreyId": "B4",
  "Offset": { "Col": 14, "Row": 0 },
  "Size":   { "Col": 1,  "Row": 2 },
  "CleanCostId": "ELEVATOR",
  "CostLabor": 0,
  "ProvideLabor": 8
}
```

`Offset.Row` is **already absolute** — it includes the storey's `YOffset`
(B4=0, B3=2, B2=4, B1=6, 1F=8), so no join is needed.

### Units are half-tiles

Rows advance in 2s and every room is `Size.Row: 2`. One visual row = 2 units.
Columns use the same scale, which reproduces adachurch's grid exactly:

| Category            | `Size.Col` | Renders as              | Adachurch class      |
|---------------------|-----------|--------------------------|----------------------|
| ELEVATOR            | 1         | 32px (narrow column)     | `.riic-tile-elevator`|
| CORRIDOR            | 2         | 64px, 1 col              | `.riic-tile-path`    |
| OUTPUT, FUNCTION    | 4         | 2 cols                   | `room-body-2x1`      |
| CUSTOM, CUSTOM_P    | 6         | 3 cols                   | `room-body-3x1`      |
| SPECIAL (control)   | 8 x 4     | 4 cols x 2 rows          | `room-body-4x2`      |

Their body-variant class names are literally `Size / 2`.

### Target `ITile`

```ts
export interface ITile {
  slotId: string;
  type: RoomType | "ELEVATOR" | "CORRIDOR";
  name: string;
  level: number;
  maxPhase: number;
  col: number;  // Offset.Col + 1
  row: number;  // Offset.Row / 2 + 1
  w: number;    // Size.Col / 2  (0.5 for elevators -> the 32px track)
  h: number;    // Size.Row / 2
}
```

`Slots` should be exposed by the backend alongside the existing catalog
(`GET /base/catalog` already reads `building_data`) rather than shipping a 5MB
JSON to the browser.

---

## Facility enum

`Layouts` aside, `building_data.json` -> `Rooms` has exactly 12 keys. Verified
mapping to adachurch's `data-facility-type` strings and to asset filename stems:

| Game key      | Name           | Adachurch string  | Asset stem  | Phases |
|---------------|----------------|-------------------|-------------|--------|
| `CONTROL`     | Control Center | `control_center`  | `Control`   | 5      |
| `POWER`       | Power Plant    | `power_plant`     | `Power`     | 3      |
| `MANUFACTURE` | Factory        | `factory`         | `Manuf`     | 3      |
| `TRADING`     | Trading Post   | `trading_post`    | `Trading`   | 3      |
| `DORMITORY`   | Dormitory      | `dormitory`       | `Dormitory` | 5      |
| `PRIVATE`     | Activity Room  | `activity_room`   | `Private`   | 3      |
| `WORKSHOP`    | Workshop       | `workshop`        | `Workshop`  | 3      |
| `HIRE`        | Office         | `office`          | `Hire`      | 3      |
| `TRAINING`    | Training Room  | `training_room`   | `training`  | 3      |
| `MEETING`     | Reception Room | `reception_room`  | `Meeting`   | 3      |
| `ELEVATOR`    | Elevator       | `elevator`        | —           | 1      |
| `CORRIDOR`    | Corridor       | `path`            | —           | 1      |

It is `TRADING`, not `SHOP`. `PRIVATE` is the Activity Room.

**Level pip count = `Phases.length`.** Confirmed against the live site: dormitory
shows 5 pips, factory/trading/reception show 3. Do not hardcode it. The control
centre shows 0 pips — it has its own badge UI instead.

---

## Reference DOM (read from the live adachurch site)

Grid container: 23 cols x 6 rows, `gap: 3px`, template
`repeat(7,64px) 32px repeat(4,64px) 32px repeat(3,64px) 32px repeat(6,64px)`.

**The 32px columns are not gutters** — they are lift shafts, at columns 8, 13 and
17, filled six deep with `.riic-tile-elevator`. Do not collapse them.

A normal room:

```html
<button class="riic-tile riic-tile-flexible"
        type="button"
        data-slot-id="production_9"
        data-facility-type="trading_post"
        title="Trading Post"
        style="grid-column: ...; grid-row: ...">
  <span class="riic-tile-room-clip">      <!-- mask-image: room_outline.svg -->
    <span class="riic-tile-room-body"></span>
    <span class="riic-tile-room-accent"></span>
    <span class="riic-tile-room-icon"></span>   <!-- mask-image: room_icon_trading_post.svg -->
  </span>
  <span class="riic-tile-info">
    <span class="riic-tile-name-row">
      <span class="riic-tile-name">Trading Post</span>
      <span class="riic-tile-level-pips">
        <span class="riic-tile-level-pip"></span>  <!-- x3 -->
      </span>
    </span>
  </span>
</button>
```

Elevators and corridors are bare `<div class="riic-tile riic-tile-elevator">`
with **no children** — pure CSS.

The control centre is `.riic-tile-fixed` with
`data-facility-type="control_center"` and a wholly different subtree:
`room-body-4x2`, `room-grid`, `room-icon-plain`, then a badge stack of
`riic-cc-badge`, `riic-cc-bracket`, `riic-cc-badge-inner`, `riic-cc-title`,
`riic-cc-subtitle`, `riic-cc-ver-row`. It needs its own component; do not force
it through `RoomTile`.

---

## Asset URL mechanics

Backend serves the extraction directly — nothing is bundled by Vite.

- `ASSETS_DIR` (`/data/output`, dev default `../assets/output`) + server (`en`/`cn`)
- Route: `/api/assets/{*path}` and `/api/{server}/assets/{*path}`
- Helper: `asset(path, server?)` in `components/operators/detail/impl/assets.ts`
- `generic_impl` falls back to the default server when a server-specific file is
  missing, so no branching is needed for the RIIC set.

Gotchas: percent-encode `[` / `]` (the corridor art lives under
`building/blueprint/[uc]rooms/`), and CSS cannot read `VITE_BACKEND_URL`, so any
`url()` has to be set inline from JS. Both disappear if the icons are vendored
into `frontend/public/riic/` — which is the recommendation, and matches the
existing `public/stat-icons/` precedent.

---

## Files to change

- `backend/src/app/routes/` — expose `Layouts[0].value.Slots` on the catalog response
- `frontend/src/lib/base/layout.ts` — add `col/row/w/h` to `ITile`; delete the
  "geometry has to come from somewhere else" comment; map slots in `toTiles`
- `frontend/src/.../Optimizer/Board.tsx` — place tiles by coordinate, drop auto-flow
- `frontend/src/.../Optimizer/RiicTile.tsx` — dispatch elevator/corridor/control/room
- `frontend/src/.../Optimizer/RoomTile.tsx` — add `.riic-tile-room-icon`; stop
  putting the icon on the tile background
- `frontend/src/.../Optimizer/Board.css` — write the ~8 missing rules
- New: `ControlCenterTile.tsx`
- New: `frontend/public/riic/*.svg`

---

## Unverified — check before relying on these

1. **Filled pip styling.** The adachurch board was fully unbuilt, so every pip
   was in the same state. `data-on` is our own convention, not theirs.
2. **Whether `icon_title_*.png` are true monochrome.** Inferred from size and
   naming; not opened.
3. **Reconciling `Slots` with the player's `roomSlots`.** The layout gives 51
   slots; the live adachurch board showed 48 tiles. Something is filtered —
   likely locked storeys. Confirm before assuming a 1:1 join.
4. **`.riic-tile-room-grid`.** Present in adachurch's stylesheet but absent from
   normal room markup; only the control centre uses it.
