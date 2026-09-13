import { BaseBoard } from "frontend";
import { type CSSProperties, type ReactNode, useEffect, useRef } from "react";

// BaseBoard (`base/board/Board.tsx`) is the RIIC floorplan: a CSS grid of room
// tiles laid out from the base catalog's slot coordinates. It takes a prebuilt
// `IBoard` - the app derives one with `buildBoard(slots, rooms, catalog, roster,
// marks)` from `#/lib/base/board`, which is not a bundle export, so the same
// derivation is ported below over the live `/api/base/catalog` slot table
// (48 slots, half-tile units: rooms are 2 units tall and an even number wide,
// elevators 1 unit wide). Only the closed board is graded here: the popover a
// tile opens (`RoomPopover`) reads the optimizer context, which no preview can
// supply.

// ---------------------------------------------------------------------------
// Catalog (live `/api/base/catalog`, 2026-09-13)
// ---------------------------------------------------------------------------

type Category = "ELEVATOR" | "CORRIDOR" | "SPECIAL" | "FUNCTION" | "OUTPUT" | "CUSTOM" | "CUSTOM_P";

// [slot_id, category, offset_col, offset_row, size_col, size_row]
const SLOTS: [string, Category, number, number, number, number][] = [
    ["slot_37", "ELEVATOR", 14, 10, 1, 2],
    ["slot_38", "ELEVATOR", 23, 10, 1, 2],
    ["slot_33", "ELEVATOR", 14, 8, 1, 2],
    ["slot_34", "SPECIAL", 15, 8, 8, 4],
    ["slot_35", "ELEVATOR", 23, 8, 1, 2],
    ["slot_36", "FUNCTION", 24, 8, 8, 2],
    ["slot_48", "ELEVATOR", 32, 8, 1, 2],
    ["slot_24", "OUTPUT", 2, 6, 4, 2],
    ["slot_25", "OUTPUT", 6, 6, 4, 2],
    ["slot_26", "OUTPUT", 10, 6, 4, 2],
    ["slot_27", "ELEVATOR", 14, 6, 1, 2],
    ["slot_28", "CUSTOM", 15, 6, 6, 2],
    ["slot_29", "CORRIDOR", 21, 6, 2, 2],
    ["slot_30", "ELEVATOR", 23, 6, 1, 2],
    ["slot_31", "CORRIDOR", 24, 6, 2, 2],
    ["slot_32", "FUNCTION", 26, 6, 4, 2],
    ["slot_45", "CORRIDOR", 30, 6, 2, 2],
    ["slot_46", "ELEVATOR", 32, 6, 1, 2],
    ["slot_47", "CUSTOM_P", 33, 6, 6, 2],
    ["slot_51", "CUSTOM_P", 39, 6, 6, 2],
    ["slot_14", "OUTPUT", 0, 4, 4, 2],
    ["slot_15", "OUTPUT", 4, 4, 4, 2],
    ["slot_16", "OUTPUT", 8, 4, 4, 2],
    ["slot_17", "CORRIDOR", 12, 4, 2, 2],
    ["slot_18", "ELEVATOR", 14, 4, 1, 2],
    ["slot_19", "CORRIDOR", 15, 4, 2, 2],
    ["slot_20", "CUSTOM", 17, 4, 6, 2],
    ["slot_21", "ELEVATOR", 23, 4, 1, 2],
    ["slot_22", "CORRIDOR", 24, 4, 2, 2],
    ["slot_23", "FUNCTION", 26, 4, 4, 2],
    ["slot_42", "CORRIDOR", 30, 4, 2, 2],
    ["slot_43", "ELEVATOR", 32, 4, 1, 2],
    ["slot_44", "CUSTOM_P", 33, 4, 6, 2],
    ["slot_50", "CUSTOM_P", 39, 4, 6, 2],
    ["slot_5", "OUTPUT", 2, 2, 4, 2],
    ["slot_6", "OUTPUT", 6, 2, 4, 2],
    ["slot_7", "OUTPUT", 10, 2, 4, 2],
    ["slot_8", "ELEVATOR", 14, 2, 1, 2],
    ["slot_9", "CUSTOM", 15, 2, 6, 2],
    ["slot_10", "CORRIDOR", 21, 2, 2, 2],
    ["slot_11", "ELEVATOR", 23, 2, 1, 2],
    ["slot_12", "CORRIDOR", 24, 2, 2, 2],
    ["slot_13", "FUNCTION", 26, 2, 4, 2],
    ["slot_39", "CORRIDOR", 30, 2, 2, 2],
    ["slot_40", "ELEVATOR", 32, 2, 1, 2],
    ["slot_41", "CUSTOM_P", 33, 2, 6, 2],
    ["slot_49", "CUSTOM_P", 39, 2, 6, 2],
    ["slot_1", "ELEVATOR", 14, 0, 1, 2],
    ["slot_2", "CORRIDOR", 15, 0, 2, 2],
    ["slot_3", "CUSTOM", 17, 0, 6, 2],
    ["slot_4", "ELEVATOR", 23, 0, 1, 2],
];

type Facility = "CONTROL" | "MANUFACTURE" | "TRADING" | "POWER" | "DORMITORY" | "MEETING" | "HIRE" | "TRAINING" | "WORKSHOP" | "PRIVATE" | "ELEVATOR" | "CORRIDOR";

// room_type -> [name, category, seats per phase]
const ROOMS: Record<Facility, [string, Category, number[]]> = {
    CONTROL: ["Control Center", "SPECIAL", [1, 2, 3, 4, 5]],
    CORRIDOR: ["Corridor", "SPECIAL", [0]],
    DORMITORY: ["Dormitory", "CUSTOM", [5, 5, 5, 5, 5]],
    ELEVATOR: ["Elevator", "SPECIAL", [0]],
    HIRE: ["Office", "FUNCTION", [1, 1, 1]],
    MANUFACTURE: ["Factory", "OUTPUT", [1, 2, 3]],
    MEETING: ["Reception Room", "FUNCTION", [2, 2, 2]],
    POWER: ["Power Plant", "OUTPUT", [1, 1, 1]],
    PRIVATE: ["Activity Room", "CUSTOM_P", [0, 0, 0]],
    TRADING: ["Trading Post", "OUTPUT", [1, 2, 3]],
    TRAINING: ["Training Room", "FUNCTION", [2, 2, 2]],
    WORKSHOP: ["Workshop", "FUNCTION", [1, 1, 1]],
};

// The one category with more buildable room kinds than slots (5 factories + 5
// trading posts + 3 plants over 9 OUTPUT slots) - those tiles are "flexible"
// and carry the room's accent stripe. Every other category is fixed.
const FLEXIBLE = new Set<Category>(["OUTPUT"]);

// A fixed slot with no room still knows what it can only ever hold.
const FIXED_FACILITY: Partial<Record<Category, Facility>> = { CUSTOM: "DORMITORY", CUSTOM_P: "PRIVATE", CORRIDOR: "CORRIDOR", ELEVATOR: "ELEVATOR" };

// ---------------------------------------------------------------------------
// buildBoard port (`#/lib/base/board.ts`)
// ---------------------------------------------------------------------------

interface IRoom {
    slot_id: string;
    room_type: Facility;
    level: number;
    operators: string[];
}

type Marks = Map<string, Map<string, "added" | "removed">>;

function measureTracks(spans: { offset: number; size: number }[]) {
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

function buildBoard(rooms: IRoom[], names: Record<string, string>, marks?: Marks) {
    const cols = measureTracks(SLOTS.map(([, , col, , w]) => ({ offset: col, size: w })));
    const rows = measureTracks(SLOTS.map(([, , , row, , h]) => ({ offset: row, size: h })));
    const rowTracks = rows.widths.length;
    const bySlot = new Map(rooms.map((room) => [room.slot_id, room]));

    const tiles = SLOTS.map(([slotId, category, offCol, offRow, sizeCol, sizeRow]) => {
        const room = bySlot.get(slotId);
        const rowStart = rows.trackAt.get(offRow) ?? 0;
        const rowEnd = rows.trackAt.get(offRow + sizeRow) ?? rowStart;
        const colStart = cols.trackAt.get(offCol) ?? 0;
        const colEnd = cols.trackAt.get(offCol + sizeCol) ?? colStart;
        const kind = category === "ELEVATOR" ? "elevator" : category === "CORRIDOR" ? "path" : FLEXIBLE.has(category) ? "flexible" : "fixed";
        const facility: Facility | null = room?.room_type ?? FIXED_FACILITY[category] ?? null;
        return {
            slotId,
            kind,
            facility,
            name: facility ? ROOMS[facility][0] : "",
            level: room?.level ?? 0,
            maxPhase: facility ? ROOMS[facility][2].length : 0,
            built: room !== undefined || kind === "elevator" || kind === "path",
            operators: (room?.operators ?? []).map((id) => ({ id, name: names[id] ?? id, skills: [], change: marks?.get(slotId)?.get(id) })),
            seats: room ? (ROOMS[room.room_type][2][room.level - 1] ?? 0) : 0,
            col: colStart + 1,
            row: rowTracks - rowEnd + 1,
            w: colEnd - colStart,
            h: rowEnd - rowStart,
        };
    });

    return { tiles, templateColumns: toTemplate(cols.widths), templateRows: toTemplate(rows.widths) };
}

// ---------------------------------------------------------------------------
// Fixtures - ids checked against /api/operators/index
// ---------------------------------------------------------------------------

const NAMES: Record<string, string> = {
    char_272_strong: "Jaye",
    char_4032_provs: "Proviso",
    char_486_takila: "Tequila",
    char_254_vodfox: "Shamare",
    char_102_texas: "Texas",
    char_140_whitew: "Lappland",
    char_103_angel: "Exusiai",
    char_277_sqrrel: "Shaw",
    char_253_greyy: "Greyy",
    char_183_skgoat: "Earthspirit",
    char_190_clour: "Vermeil",
    char_336_folivo: "Scene",
    char_163_hpsts: "Vulcan",
    char_400_weedy: "Weedy",
    char_196_sunbr: "Gummy",
    char_4105_almond: "Almond",
    char_128_plosis: "Ptilopsis",
    char_2014_nian: "Nian",
    char_2015_dusk: "Dusk",
    char_478_kirara: "Kirara",
    char_252_bibeak: "Bibeak",
    char_416_zumama: "Eunectes",
    char_002_amiya: "Amiya",
    char_308_swire: "Swire",
    char_003_kalts: "Kal'tsit",
    char_130_doberm: "Dobermann",
    char_010_chen: "Ch'en",
    char_101_sora: "Sora",
    char_109_fmout: "Gitano",
    char_4080_lin: "Lin",
    char_108_silent: "Silence",
    char_265_sophia: "Whislash",
    char_213_mostma: "Mostima",
    char_300_phenxi: "Fiammetta",
    char_285_medic2: "Lancet-2",
    char_180_amgoat: "Eyjafjalla",
    char_129_bluep: "Blue Poison",
    char_181_flower: "Perfumer",
    char_212_ansel: "Ansel",
    char_278_orchid: "Orchid",
    char_240_wyvern: "Vanilla",
    char_209_ardign: "Cardigan",
    char_304_zebra: "Heavyrain",
    char_433_windft: "Windflit",
    char_2023_ling: "Ling",
    char_388_mint: "Mint",
    char_402_tuye: "Tuye",
    char_348_ceylon: "Ceylon",
};

const room = (slot_id: string, room_type: Facility, level: number, operators: string[] = []): IRoom => ({ slot_id, room_type, level, operators });

// A mature "2-4-3" base: two trading posts, four factories, three plants,
// every dorm built, Control Center at 5.
const MATURE_BASE: IRoom[] = [
    room("slot_34", "CONTROL", 5, ["char_002_amiya", "char_308_swire", "char_003_kalts", "char_130_doberm", "char_010_chen"]),
    room("slot_24", "TRADING", 3, ["char_272_strong", "char_4032_provs", "char_486_takila"]),
    room("slot_25", "TRADING", 3, ["char_254_vodfox", "char_102_texas", "char_140_whitew"]),
    room("slot_26", "POWER", 3, ["char_277_sqrrel"]),
    room("slot_14", "MANUFACTURE", 3, ["char_190_clour", "char_336_folivo", "char_163_hpsts"]),
    room("slot_15", "MANUFACTURE", 3, ["char_400_weedy", "char_196_sunbr", "char_4105_almond"]),
    room("slot_16", "POWER", 3, ["char_253_greyy"]),
    room("slot_5", "MANUFACTURE", 3, ["char_128_plosis", "char_2014_nian", "char_2015_dusk"]),
    room("slot_6", "MANUFACTURE", 3, ["char_478_kirara", "char_252_bibeak", "char_416_zumama"]),
    room("slot_7", "POWER", 3, ["char_183_skgoat"]),
    room("slot_36", "MEETING", 3, ["char_101_sora", "char_109_fmout"]),
    room("slot_32", "HIRE", 3, ["char_4080_lin"]),
    room("slot_23", "WORKSHOP", 3, ["char_108_silent"]),
    room("slot_13", "TRAINING", 2, ["char_265_sophia", "char_213_mostma"]),
    room("slot_28", "DORMITORY", 5, ["char_300_phenxi", "char_285_medic2", "char_180_amgoat", "char_129_bluep", "char_181_flower"]),
    room("slot_20", "DORMITORY", 3, ["char_212_ansel", "char_278_orchid", "char_240_wyvern"]),
    room("slot_9", "DORMITORY", 3, ["char_209_ardign", "char_304_zebra", "char_433_windft", "char_2023_ling", "char_388_mint"]),
    room("slot_3", "DORMITORY", 1, ["char_402_tuye", "char_348_ceylon"]),
];

// The rotation view of the same base for one shift: a tile lists the incoming
// crew first and the outgoing operators after it, and `change` marks each.
const HANDOVER_BASE: IRoom[] = MATURE_BASE.map((r) => {
    switch (r.slot_id) {
        case "slot_25":
            return { ...r, operators: ["char_254_vodfox", "char_102_texas", "char_103_angel", "char_140_whitew"] };
        case "slot_14":
            return { ...r, operators: ["char_190_clour", "char_336_folivo", "char_4105_almond", "char_163_hpsts"] };
        case "slot_15":
            return { ...r, operators: ["char_400_weedy", "char_196_sunbr"] };
        case "slot_28":
            return { ...r, operators: ["char_300_phenxi", "char_140_whitew", "char_163_hpsts", "char_129_bluep", "char_181_flower", "char_285_medic2", "char_180_amgoat"] };
        default:
            return r;
    }
});

const HANDOVER_MARKS: Marks = new Map([
    ["slot_25", new Map<string, "added" | "removed">([["char_103_angel", "added"], ["char_140_whitew", "removed"]])],
    ["slot_14", new Map<string, "added" | "removed">([["char_4105_almond", "added"], ["char_163_hpsts", "removed"]])],
    ["slot_28", new Map<string, "added" | "removed">([["char_140_whitew", "added"], ["char_163_hpsts", "added"], ["char_285_medic2", "removed"], ["char_180_amgoat", "removed"]])],
]);

// A fresh account: Control Center 2, so only B1 and 1F are open.
const EARLY_BASE: IRoom[] = [
    room("slot_34", "CONTROL", 2, ["char_002_amiya", "char_130_doberm"]),
    room("slot_24", "TRADING", 1, ["char_102_texas"]),
    room("slot_25", "MANUFACTURE", 1, ["char_190_clour"]),
    room("slot_26", "POWER", 1, ["char_277_sqrrel"]),
    room("slot_28", "DORMITORY", 1, ["char_212_ansel", "char_278_orchid", "char_240_wyvern"]),
    room("slot_36", "MEETING", 1, ["char_101_sora"]),
];

// ---------------------------------------------------------------------------
// Stage - the scroll container `BasePanel` mounts the board in (its CSS-module
// class is not reachable from a preview, so the same rules are inlined).
// ---------------------------------------------------------------------------

const STAGE: CSSProperties = {
    position: "relative",
    overflow: "auto",
    borderRadius: "calc(var(--radius) - 2px)",
    backgroundColor: "#191919",
    backgroundImage: "repeating-linear-gradient(-45deg, rgb(255 255 255 / 0.016) 0 6px, transparent 6px 12px)",
};

// The board is ~1,240px wide at its smallest container step; the app scrolls it
// and centres on the Control Center. `zoom` mirrors the fullscreen pan view.
// (`.riic-board-fit` is an inline-size container, so it must stay a block child:
// as a flex item its inline size collapses and the board overflows to the right.)
function Stage({ children, zoom, centerOnControl }: { children: ReactNode; zoom?: number; centerOnControl?: boolean }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        if (!centerOnControl) return;
        const frame = requestAnimationFrame(() => {
            const el = ref.current;
            if (!el) return;
            el.scrollLeft = Math.max(0, (el.scrollWidth - el.clientWidth) / 2);
            const cc = el.querySelector<HTMLElement>('[data-facility-type="CONTROL"]');
            if (!cc) return;
            const view = el.getBoundingClientRect();
            const rect = cc.getBoundingClientRect();
            el.scrollLeft = Math.max(0, el.scrollLeft + rect.left + rect.width / 2 - (view.left + view.width / 2));
        });
        return () => cancelAnimationFrame(frame);
    }, [centerOnControl]);
    return (
        <div className="rounded-xl border border-border bg-card p-3">
            <div ref={ref} style={STAGE}>
                <div style={zoom ? { zoom } : undefined}>{children}</div>
            </div>
        </div>
    );
}

/** The board as `BasePanel` shows it: a scrollable stage centred on the Control Center. */
export const StaffedBase = () => (
    <Stage centerOnControl>
        <BaseBoard board={buildBoard(MATURE_BASE, NAMES)} />
    </Stage>
);

/** The whole floorplan at once, as the fullscreen pan view zooms it. */
export const WholeFloorplan = () => (
    <Stage zoom={0.66}>
        <BaseBoard board={buildBoard(MATURE_BASE, NAMES)} />
    </Stage>
);

/** A rotation shift: incoming operators ring in the room's accent, outgoing ones fade and dash. */
export const ShiftHandover = () => (
    <Stage zoom={0.66}>
        <BaseBoard board={buildBoard(HANDOVER_BASE, NAMES, HANDOVER_MARKS)} />
    </Stage>
);

/** A new account: six rooms built, every other slot empty or "Not Built". */
export const EarlyBase = () => (
    <Stage zoom={0.66}>
        <BaseBoard board={buildBoard(EARLY_BASE, NAMES)} />
    </Stage>
);
