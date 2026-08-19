import type { FacilityType, ICatalogSlot, IDraftRoom } from "#/lib/api/base";
import { type Catalog, roomLabel, seatsOf } from "./catalog";
import type { IRosterOption, IRosterSkill } from "./roster";

type TileKind = "fixed" | "flexible" | "path" | "elevator";

export interface ITileOperator {
    id: string;
    name: string;
    skills: IRosterSkill[];
    change?: "added" | "removed";
}

export type BoardMarks = Map<string, Map<string, "added" | "removed">>;

export interface ITile {
    slotId: string;
    kind: TileKind;
    facility: FacilityType | null;
    name: string;
    level: number;
    maxPhase: number;
    built: boolean;
    operators: ITileOperator[];
    seats: number;
    col: number;
    row: number;
    w: number;
    h: number;
}

export function vacanciesOf(tile: ITile): number {
    return Math.max(0, tile.seats - tile.operators.length);
}

export interface IBoard {
    tiles: ITile[];
    templateColumns: string;
    templateRows: string;
}

const HALF_TILE = "var(--riic-unit)";

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
    trackAt.set(extent, widths.length);

    return { trackAt, widths };
}

const toTemplate = (widths: number[]): string => widths.map((w) => `calc(${w} * ${HALF_TILE})`).join(" ");

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

export function buildBoard(slots: ICatalogSlot[], rooms: IDraftRoom[], catalog: Catalog, roster: Map<string, IRosterOption> = new Map(), marks?: BoardMarks): IBoard {
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
            maxPhase: facility ? (catalog.get(facility)?.phases.length ?? room?.level ?? 0) : 0,
            built: room !== undefined || kind === "elevator" || kind === "path",
            operators: (room?.operators ?? []).map((id) => {
                const member = roster.get(id);
                return {
                    id,
                    name: member?.name ?? id,
                    skills: room ? (member?.baseSkills ?? []).filter((skill) => skill.roomType === room.room_type) : [],
                    change: marks?.get(slot.slot_id)?.get(id),
                };
            }),
            seats: room ? seatsOf(room, catalog) : 0,
            col: colStart + 1,
            row: rowTracks - rowEnd + 1,
            w: colEnd - colStart,
            h: rowEnd - rowStart,
        };
    });

    return { tiles, templateColumns: toTemplate(cols.widths), templateRows: toTemplate(rows.widths) };
}

function facilityForCategory(category: string, catalog: Catalog, slotCategories: Set<string>): FacilityType | null {
    if (catalog.has(category as FacilityType)) return category as FacilityType;

    const candidates = [...catalog.values()].filter((def) => def.category === category && !slotCategories.has(def.room_type));
    return candidates.length === 1 ? candidates[0].room_type : null;
}
