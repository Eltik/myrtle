import { BaseBoard, RiicTile } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/*
 * RiicTile is the board's dispatcher: an elevator or corridor slot renders as
 * a bare shaft / plate, the CONTROL slot as `ControlCenterTile`, everything
 * else as `RoomTile`. The floorplan below is the real one from
 * `GET /api/base/catalog` (slot ids, half-tile offsets and sizes), cut to the
 * left wing so it fits a card; the tiles are built the way
 * `buildBoard()` in `src/lib/base/board.ts` builds them.
 */
type Kind = "fixed" | "flexible" | "path" | "elevator";
type Facility = "TRADING" | "MANUFACTURE" | "POWER" | "DORMITORY" | "CONTROL" | "MEETING" | "HIRE" | "ELEVATOR" | "CORRIDOR";
interface ICrew {
    id: string;
    name: string;
    skills: never[];
    change?: "added" | "removed";
}
interface ITile {
    slotId: string;
    kind: Kind;
    facility: Facility | null;
    name: string;
    level: number;
    maxPhase: number;
    built: boolean;
    operators: ICrew[];
    seats: number;
    col: number;
    row: number;
    w: number;
    h: number;
}

const ROOMS: Record<Facility, { name: string; seats: number[] }> = {
    TRADING: { name: "Trading Post", seats: [1, 2, 3] },
    MANUFACTURE: { name: "Factory", seats: [1, 2, 3] },
    POWER: { name: "Power Plant", seats: [1, 1, 1] },
    DORMITORY: { name: "Dormitory", seats: [5, 5, 5, 5, 5] },
    CONTROL: { name: "Control Center", seats: [1, 2, 3, 4, 5] },
    MEETING: { name: "Reception Room", seats: [2, 2, 2] },
    HIRE: { name: "Office", seats: [1, 1, 1] },
    ELEVATOR: { name: "Elevator", seats: [0] },
    CORRIDOR: { name: "Corridor", seats: [0] },
};

const op = (id: string, name: string): ICrew => ({ id, name, skills: [] });

const CREW: Record<string, ICrew[]> = {
    slot_5: [op("char_272_strong", "Jaye"), op("char_1028_texas2", "Texas the Omertosa"), op("char_140_whitew", "Lappland")],
    slot_6: [op("char_190_clour", "Vermeil"), op("char_196_sunbr", "Gummy"), op("char_440_pinecn", "Pinecone")],
    slot_7: [op("char_253_greyy", "Greyy")],
    slot_9: [op("char_212_ansel", "Ansel"), op("char_120_hibisc", "Hibiscus"), op("char_278_orchid", "Orchid"), op("char_181_flower", "Perfumer"), op("char_237_gravel", "Gravel")],
    slot_14: [op("char_4032_provs", "Proviso"), op("char_486_takila", "Tequila"), op("char_254_vodfox", "Shamare")],
    slot_15: [op("char_163_hpsts", "Vulcan"), op("char_130_doberm", "Dobermann"), op("char_437_mizuki", "Mizuki")],
    slot_16: [op("char_277_sqrrel", "Shaw")],
    slot_20: [op("char_348_ceylon", "Ceylon"), op("char_436_whispr", "Whisperain"), op("char_108_silent", "Silence")],
    slot_24: [op("char_2013_cerber", "Ceobe"), op("char_431_ashlok", "Ashlock"), op("char_284_spot", "Spot")],
    slot_25: [op("char_493_firwhl", "Firewhistle")],
    slot_26: [op("char_285_medic2", "Lancet-2")],
    slot_28: [op("char_235_jesica", "Jessica"), op("char_243_waaifu", "Waai Fu"), op("char_263_skadi", "Skadi"), op("char_171_bldsk", "Warfarin")],
    slot_34: [op("char_002_amiya", "Amiya"), op("char_003_kalts", "Kal'tsit"), op("char_172_svrash", "SilverAsh"), op("char_308_swire", "Swire"), op("char_010_chen", "Ch'en")],
};

/** `[slot_id, category, offset_col, offset_row, size_col, size_row]` - the catalog's left wing: B4 up to the Control Center storey. */
const SLOTS: Array<[string, string, number, number, number, number]> = [
    ["slot_37", "ELEVATOR", 14, 10, 1, 2],
    ["slot_38", "ELEVATOR", 23, 10, 1, 2],
    ["slot_33", "ELEVATOR", 14, 8, 1, 2],
    ["slot_34", "SPECIAL", 15, 8, 8, 4],
    ["slot_35", "ELEVATOR", 23, 8, 1, 2],
    ["slot_24", "OUTPUT", 2, 6, 4, 2],
    ["slot_25", "OUTPUT", 6, 6, 4, 2],
    ["slot_26", "OUTPUT", 10, 6, 4, 2],
    ["slot_27", "ELEVATOR", 14, 6, 1, 2],
    ["slot_28", "CUSTOM", 15, 6, 6, 2],
    ["slot_29", "CORRIDOR", 21, 6, 2, 2],
    ["slot_30", "ELEVATOR", 23, 6, 1, 2],
    ["slot_14", "OUTPUT", 0, 4, 4, 2],
    ["slot_15", "OUTPUT", 4, 4, 4, 2],
    ["slot_16", "OUTPUT", 8, 4, 4, 2],
    ["slot_17", "CORRIDOR", 12, 4, 2, 2],
    ["slot_18", "ELEVATOR", 14, 4, 1, 2],
    ["slot_19", "CORRIDOR", 15, 4, 2, 2],
    ["slot_20", "CUSTOM", 17, 4, 6, 2],
    ["slot_21", "ELEVATOR", 23, 4, 1, 2],
    ["slot_5", "OUTPUT", 2, 2, 4, 2],
    ["slot_6", "OUTPUT", 6, 2, 4, 2],
    ["slot_7", "OUTPUT", 10, 2, 4, 2],
    ["slot_8", "ELEVATOR", 14, 2, 1, 2],
    ["slot_9", "CUSTOM", 15, 2, 6, 2],
    ["slot_10", "CORRIDOR", 21, 2, 2, 2],
    ["slot_11", "ELEVATOR", 23, 2, 1, 2],
    ["slot_1", "ELEVATOR", 14, 0, 1, 2],
    ["slot_2", "CORRIDOR", 15, 0, 2, 2],
    ["slot_3", "CUSTOM", 17, 0, 6, 2],
    ["slot_4", "ELEVATOR", 23, 0, 1, 2],
];

/** The player's draft: `[room_type, level]` per built slot. `slot_3` (B4's dorm) is left unbuilt. */
const DRAFT: Record<string, [Facility, number]> = {
    slot_5: ["TRADING", 3],
    slot_6: ["MANUFACTURE", 3],
    slot_7: ["POWER", 3],
    slot_9: ["DORMITORY", 5],
    slot_14: ["TRADING", 3],
    slot_15: ["MANUFACTURE", 3],
    slot_16: ["POWER", 3],
    slot_20: ["DORMITORY", 4],
    slot_24: ["MANUFACTURE", 3],
    slot_25: ["MANUFACTURE", 2],
    slot_26: ["POWER", 3],
    slot_28: ["DORMITORY", 5],
    slot_34: ["CONTROL", 5],
};

/** `measureTracks` from `src/lib/base/board.ts`: half-tile units -> grid tracks, elevators as 1-unit tracks. */
function measureTracks(spans: Array<{ offset: number; size: number }>) {
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

const toTemplate = (widths: number[]) => widths.map((w) => `calc(${w} * var(--riic-unit))`).join(" ");

function facilityFor(slotId: string, category: string): Facility | null {
    const drafted = DRAFT[slotId];
    if (drafted) return drafted[0];
    if (category === "ELEVATOR" || category === "CORRIDOR") return category;
    if (category === "CUSTOM") return "DORMITORY";
    return null;
}

function buildBoard() {
    const cols = measureTracks(SLOTS.map(([, , c, , w]) => ({ offset: c, size: w })));
    const rows = measureTracks(SLOTS.map(([, , , r, , h]) => ({ offset: r, size: h })));
    const rowTracks = rows.widths.length;
    const tiles = SLOTS.map(([slotId, category, c, r, w, h]): ITile => {
        const drafted = DRAFT[slotId];
        const facility = facilityFor(slotId, category);
        const kind: Kind = category === "ELEVATOR" ? "elevator" : category === "CORRIDOR" ? "path" : category === "OUTPUT" ? "flexible" : "fixed";
        const rowStart = rows.trackAt.get(r) ?? 0;
        const rowEnd = rows.trackAt.get(r + h) ?? rowStart;
        const colStart = cols.trackAt.get(c) ?? 0;
        const colEnd = cols.trackAt.get(c + w) ?? colStart;
        const level = drafted?.[1] ?? 0;
        return {
            slotId,
            kind,
            facility,
            name: facility ? ROOMS[facility].name : "",
            level,
            maxPhase: facility ? ROOMS[facility].seats.length : 0,
            built: drafted !== undefined || kind === "elevator" || kind === "path",
            operators: CREW[slotId] ?? [],
            seats: drafted ? ROOMS[drafted[0]].seats[level - 1] : 0,
            col: colStart + 1,
            row: rowTracks - rowEnd + 1,
            w: colEnd - colStart,
            h: rowEnd - rowStart,
        };
    });
    return { tiles, templateColumns: toTemplate(cols.widths), templateRows: toTemplate(rows.widths) };
}

const BOARD = buildBoard();

/** Every kind at once, in `BaseBoard` (the real parent): the left wing from B4 up to the Control Center, elevator shafts as half-width columns. */
export const LeftWing = () => (
    // `.riic-board-fit` is a size container, so it needs a definite width from its parent (the scroll pane in-app).
    <div style={{ width: "100%", background: "#191919", borderRadius: 6 }}>
        <BaseBoard board={BOARD} />
    </div>
);

const Stage = ({ columns, children }: { columns: string; children: ReactNode }) => (
    <div style={{ width: "fit-content", borderRadius: 6, background: "#191919", padding: "22px 16px" }}>
        <div style={{ "--riic-unit": "32px", display: "grid", gap: 3, width: "max-content", gridTemplateColumns: columns, gridAutoRows: "calc(2 * var(--riic-unit))" } as CSSProperties}>{children}</div>
    </div>
);

const at = (slotId: string): ITile => {
    const tile = BOARD.tiles.find((t) => t.slotId === slotId);
    if (!tile) throw new Error(`no fixture for ${slotId}`);
    return tile;
};

/** One row of B3 re-placed by hand: room tile, elevator shaft, dorm, corridor plate, shaft - each kind's own render. */
export const Kinds = () => (
    <div className="flex flex-col gap-2">
        <Stage columns="calc(2 * var(--riic-unit)) calc(2 * var(--riic-unit)) calc(1 * var(--riic-unit)) calc(2 * var(--riic-unit)) calc(2 * var(--riic-unit)) calc(2 * var(--riic-unit)) calc(2 * var(--riic-unit)) calc(1 * var(--riic-unit))">
            <RiicTile tile={{ ...at("slot_7"), col: 1, row: 1 }} />
            <RiicTile tile={{ ...at("slot_8"), col: 3, row: 1 }} />
            <RiicTile tile={{ ...at("slot_9"), col: 4, row: 1 }} />
            <RiicTile tile={{ ...at("slot_10"), col: 7, row: 1 }} />
            <RiicTile tile={{ ...at("slot_11"), col: 8, row: 1 }} />
        </Stage>
        <p className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.1em]">flexible room · elevator · fixed dorm · corridor · elevator</p>
    </div>
);
