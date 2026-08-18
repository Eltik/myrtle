import type { FacilityType, ICatalogRoom, ICatalogSlot, IDraftRoom } from "#/lib/api/base";

/**
 * Facility helpers for the base planner, all derived from the catalogue
 * (`GET /base/catalog`) rather than a local table of game constants.
 */

/** The facility catalogue, keyed by type. All twelve facilities, not just the staffable nine. */
export type Catalog = Map<FacilityType, ICatalogRoom>;

/**
 * Can this room type ever hold an operator?
 *
 * A real base's `roomSlots` includes its structure - corridors, elevators, and
 * activity rooms - which are built, levelled, and completely unplannable: zero
 * seats at every level, zero electricity, and no buff in `building_data`
 * targets them.
 *
 * Derived from the catalogue rather than a hardcoded name list, so a room type
 * the game makes staffable later starts appearing on its own.
 */
export function isPlannable(roomType: string, catalog: Catalog): boolean {
    const def = catalog.get(roomType as FacilityType);
    // Unknown until the catalogue loads - assume plannable rather than blink
    // real rooms off the board.
    if (!def) return true;
    return def.phases.some((phase) => phase.max_stationed > 0);
}

/** Seats this room has at its current level, from the real facility catalogue. */
export function seatsOf(room: IDraftRoom, catalog: Catalog): number {
    return catalog.get(room.room_type)?.phases[room.level - 1]?.max_stationed ?? 0;
}

/**
 * Display name for a room type, from the catalogue rather than a local table.
 * Takes a plain string because assignment payloads type `room_type` loosely -
 * an unrecognised type falls back to its own id rather than blanking the label.
 */
export function roomLabel(roomType: string, catalog: Catalog): string {
    return catalog.get(roomType as FacilityType)?.name ?? roomType;
}

/** Only production rooms carry a per-room yield worth showing. */
export function isProduction(roomType: string): boolean {
    return roomType === "MANUFACTURE" || roomType === "TRADING";
}

/** Power this room draws (negative) or generates (positive) at its level. */
export function powerOf(room: IDraftRoom, catalog: Catalog): number {
    return catalog.get(room.room_type)?.phases[room.level - 1]?.electricity ?? 0;
}

// ─── The board ───────────────────────────────────────────────────────────────

/**
 * What a tile draws as, following the game's own division of the floorplan.
 *
 * `flexible` is a slot the player chooses the facility for; `fixed` is one the
 * game assigns. `path` and `elevator` are the structure between them - drawn,
 * never staffed.
 */
export type TileKind = "fixed" | "flexible" | "path" | "elevator";

/** One cell of the RIIC board, floorplan geometry joined to the player's draft. */
export interface ITile {
    slotId: string;
    kind: TileKind;
    /** What to draw. `null` for a slot standing empty. */
    facility: FacilityType | null;
    name: string;
    /** `0` where levels are meaningless. */
    level: number;
    /** Levels this facility can reach; drives the pip count. */
    maxPhase: number;
    /** False for a slot the floorplan has but this player has not excavated. */
    built: boolean;
    /** 1-based CSS grid placement, in tracks. */
    col: number;
    row: number;
    w: number;
    h: number;
}

/** The board, ready to render: tiles placed, plus the grid they sit in. */
export interface IBoard {
    tiles: ITile[];
    /** A `grid-template-columns` value. */
    templateColumns: string;
    /** A `grid-template-rows` value. */
    templateRows: string;
}

/**
 * Track sizes are emitted in half-tiles rather than pixels, against a
 * `--riic-unit` the stylesheet owns. The board is a fixed-proportion schematic
 * roughly 1500px across, so on a normal page it would always overflow; letting
 * CSS set the unit lets it shrink to fit its container without this module
 * needing to know anything about the viewport.
 */
const HALF_TILE = "var(--riic-unit)";

/**
 * Map every half-tile boundary to a grid track index.
 *
 * The floorplan measures in half-tiles, but the board's tracks are not uniform:
 * a lift shaft is one half-tile wide where every room is two, which is what
 * gives the base its narrow columns. So the tracks are read off the data -
 * wherever a 1-unit slot starts, that unit is its own track - rather than
 * hardcoding a template that would silently drift if the game added a floor.
 */
function measureTracks(spans: Array<{ offset: number; size: number }>): { trackAt: Map<number, number>; widths: number[] } {
    const extent = spans.reduce((max, s) => Math.max(max, s.offset + s.size), 0);
    const narrow = new Set(spans.filter((s) => s.size === 1).map((s) => s.offset));

    const trackAt = new Map<number, number>();
    const widths: number[] = [];
    for (let unit = 0; unit < extent; ) {
        trackAt.set(unit, widths.length);
        const width = narrow.has(unit) ? 1 : 2;
        widths.push(width);
        unit += width;
    }
    // The closing boundary, so a span's width is a subtraction rather than a search.
    trackAt.set(extent, widths.length);

    return { trackAt, widths };
}

const toTemplate = (widths: number[]): string => widths.map((w) => `calc(${w} * ${HALF_TILE})`).join(" ");

/**
 * Which slot categories let the player pick what goes in them.
 *
 * Derived rather than named: a category is flexible exactly when the rooms that
 * may occupy it could not all be built at once - the game allows five factories
 * and five trading posts across nine production slots, so those slots are a
 * choice, while four function rooms across four function slots are not.
 * Structure is excluded from the tally by the same zero-seat test everything
 * else uses.
 */
function flexibleCategories(catalog: Catalog, slots: ICatalogSlot[]): Set<string> {
    const slotCount = new Map<string, number>();
    for (const slot of slots) slotCount.set(slot.category, (slotCount.get(slot.category) ?? 0) + 1);

    const capacity = new Map<string, number>();
    for (const def of catalog.values()) {
        if (!def.phases.some((phase) => phase.max_stationed > 0)) continue;
        capacity.set(def.category, (capacity.get(def.category) ?? 0) + def.max_count);
    }

    const flexible = new Set<string>();
    for (const [category, count] of capacity) {
        if (count > (slotCount.get(category) ?? 0)) flexible.add(category);
    }
    return flexible;
}

/**
 * Build the board: place every slot of the floorplan, and hang the player's
 * rooms on the slots that hold one.
 *
 * The floorplan is the source of geometry and the base is the source of
 * contents - they are joined on `slot_id`. Iterating slots rather than rooms is
 * what makes an unexcavated slot render as an empty cell instead of vanishing
 * and collapsing the grid around it.
 */
export function buildBoard(slots: ICatalogSlot[], rooms: IDraftRoom[], catalog: Catalog): IBoard {
    if (slots.length === 0) return { tiles: [], templateColumns: "", templateRows: "" };

    const cols = measureTracks(slots.map((s) => ({ offset: s.offset_col, size: s.size_col })));
    const rows = measureTracks(slots.map((s) => ({ offset: s.offset_row, size: s.size_row })));
    const rowTracks = rows.widths.length;

    const bySlot = new Map(rooms.map((room) => [room.slot_id, room]));
    const flexible = flexibleCategories(catalog, slots);
    const slotCategories = new Set(slots.map((slot) => slot.category));

    const tiles = slots.map((slot): ITile => {
        const room = bySlot.get(slot.slot_id);
        // `offset_row` counts up from B4, but B4 is the bottom of the base and
        // grid rows count down from the top, so the axis is flipped.
        const rowStart = rows.trackAt.get(slot.offset_row) ?? 0;
        const rowEnd = rows.trackAt.get(slot.offset_row + slot.size_row) ?? rowStart;
        const colStart = cols.trackAt.get(slot.offset_col) ?? 0;
        const colEnd = cols.trackAt.get(slot.offset_col + slot.size_col) ?? colStart;

        const kind: TileKind = slot.category === "ELEVATOR" ? "elevator" : slot.category === "CORRIDOR" ? "path" : flexible.has(slot.category) ? "flexible" : "fixed";
        const facility = room?.room_type ?? facilityForCategory(slot.category, catalog, slotCategories);

        return {
            slotId: slot.slot_id,
            kind,
            facility,
            name: facility ? roomLabel(facility, catalog) : "",
            level: room?.level ?? 0,
            // An unloaded catalogue should not draw a room as maxed or as empty.
            maxPhase: facility ? (catalog.get(facility)?.phases.length ?? room?.level ?? 0) : 0,
            built: room !== undefined || kind === "elevator" || kind === "path",
            col: colStart + 1,
            row: rowTracks - rowEnd + 1,
            w: colEnd - colStart,
            h: rowEnd - rowStart,
        };
    });

    return { tiles, templateColumns: toTemplate(cols.widths), templateRows: toTemplate(rows.widths) };
}

/**
 * What an empty slot of this category can only be.
 *
 * Two vocabularies nearly overlap here: a slot's `category` is usually a room
 * category (`CUSTOM_P` -> Activity Room), but the lift shafts and corridors
 * name their facility outright, because as *rooms* they are filed under
 * `SPECIAL` alongside the Control Center.
 *
 * Answers only when the category admits exactly one facility. A production slot
 * could be any of three and a function slot any of four, so naming one of them
 * would be a guess dressed up as a label - those stay empty until the base says
 * otherwise.
 *
 * The lift shafts and corridors have to be discounted when counting, or they
 * would make `SPECIAL` look ambiguous: as rooms they share that category with
 * the Control Center, but no board slot of that category ever holds one. They
 * are recognisable as the facilities that are themselves slot categories.
 */
function facilityForCategory(category: string, catalog: Catalog, slotCategories: Set<string>): FacilityType | null {
    if (catalog.has(category as FacilityType)) return category as FacilityType;

    const candidates = [...catalog.values()].filter((def) => def.category === category && !slotCategories.has(def.room_type));
    return candidates.length === 1 ? candidates[0].room_type : null;
}
