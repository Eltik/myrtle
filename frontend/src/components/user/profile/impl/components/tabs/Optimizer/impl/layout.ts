import type { ICatalogRoom, IDraftRoom, RoomType } from "#/lib/api/base";

/**
 * How the board is arranged.
 *
 * The game's floor plan is not in `building_data` - room definitions carry a
 * size, but nothing says which slot sits where. Rather than invent coordinates
 * that would quietly be wrong, the board groups the player's real rooms into
 * bands by what they do. Every room in the layout lands in exactly one band, so
 * a base with unusual room counts still renders completely.
 */

export interface BoardBand {
    id: string;
    label: string;
    /** Room types that belong to this band, in display order. */
    types: RoomType[];
}

export const BOARD_BANDS: BoardBand[] = [
    { id: "hub", label: "Control", types: ["CONTROL"] },
    { id: "production", label: "Production", types: ["MANUFACTURE", "TRADING", "POWER"] },
    { id: "facilities", label: "Facilities", types: ["MEETING", "WORKSHOP", "HIRE", "TRAINING"] },
    { id: "dorms", label: "Dormitories", types: ["DORMITORY"] },
];

export interface BandedRooms {
    band: BoardBand;
    rooms: IDraftRoom[];
}

/**
 * Slot the layout into bands. Rooms whose type no band claims are collected
 * under "Other" rather than dropped - a silently missing room is worse than an
 * ugly one.
 */
export function groupIntoBands(layout: IDraftRoom[]): BandedRooms[] {
    const claimed = new Set<string>();
    const bands: BandedRooms[] = BOARD_BANDS.map((band) => {
        const rooms = band.types.flatMap((type) => {
            const matching = layout.filter((r) => r.room_type === type);
            for (const r of matching) claimed.add(r.slot_id);
            return matching.sort(compareSlots);
        });
        return { band, rooms };
    }).filter((b) => b.rooms.length > 0);

    const leftovers = layout.filter((r) => !claimed.has(r.slot_id)).sort(compareSlots);
    if (leftovers.length > 0) {
        bands.push({ band: { id: "other", label: "Other", types: [] }, rooms: leftovers });
    }
    return bands;
}

/** Slot ids are `slot_<n>`; sort them numerically so the board keeps game order. */
function compareSlots(a: IDraftRoom, b: IDraftRoom): number {
    const na = slotNumber(a.slot_id);
    const nb = slotNumber(b.slot_id);
    if (na !== null && nb !== null) return na - nb;
    return a.slot_id.localeCompare(b.slot_id);
}

function slotNumber(slotId: string): number | null {
    const m = /(\d+)$/.exec(slotId);
    return m ? Number(m[1]) : null;
}

/** Seats this room has at its current level, from the real facility catalogue. */
export function seatsOf(room: IDraftRoom, catalog: Map<RoomType, ICatalogRoom>): number {
    return catalog.get(room.room_type)?.phases[room.level - 1]?.max_stationed ?? 0;
}

/**
 * Display name for a room type, from the catalogue rather than a local table.
 * Takes a plain string because assignment payloads type `room_type` loosely -
 * an unrecognised type falls back to its own id rather than blanking the label.
 */
export function roomLabel(roomType: string, catalog: Map<RoomType, ICatalogRoom>): string {
    return catalog.get(roomType as RoomType)?.name ?? roomType;
}

/**
 * Accent colour per room type. Purely presentational, and the one thing here
 * that is genuinely a design choice rather than game data.
 */
export const ROOM_ACCENT: Record<string, string> = {
    CONTROL: "var(--chart-1)",
    MANUFACTURE: "var(--chart-2)",
    TRADING: "var(--chart-3)",
    POWER: "var(--chart-4)",
    DORMITORY: "var(--chart-5)",
    MEETING: "var(--chart-1)",
    HIRE: "var(--chart-3)",
    TRAINING: "var(--chart-4)",
    WORKSHOP: "var(--chart-2)",
};

/**
 * Can this room type ever hold an operator?
 *
 * A real base's `roomSlots` includes its structure - corridors, elevators, and
 * activity rooms - which are built, levelled, and completely unplannable: zero
 * seats at every level, zero electricity, and no buff in `building_data`
 * targets them. Rendering them buries the nine rooms that matter under dozens
 * that can never be staffed.
 *
 * Derived from the catalogue rather than a hardcoded name list, so a room type
 * the game makes staffable later starts appearing on its own.
 */
export function isPlannable(roomType: string, catalog: Map<RoomType, ICatalogRoom>): boolean {
    const def = catalog.get(roomType as RoomType);
    // Unknown until the catalogue loads - assume plannable rather than blink
    // real rooms off the board.
    if (!def) return true;
    return def.phases.some((phase) => phase.max_stationed > 0);
}

/** Only production rooms carry a per-room yield worth showing on the board. */
export function isProduction(roomType: string): boolean {
    return roomType === "MANUFACTURE" || roomType === "TRADING";
}
