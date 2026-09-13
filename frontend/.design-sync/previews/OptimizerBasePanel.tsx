import { BaseOptimizerProvider, OptimizerBasePanel } from "frontend";

// OptimizerBasePanel (`base/BasePanel.tsx`) is the whole RIIC planner surface
// under the profile's Optimizer tab: the headline strip (efficiency, LMD and
// EXP per day, power), the Optimize / Reset actions, the shift strip, the
// promotion toggle and account facts, the board in its scroll stage (centred
// on the Control Center by the panel itself), the fullscreen dialog (closed),
// the Deep Dive (open by default) and Stats for Nerds (collapsed). Every
// number comes from the optimizer context, so each story mounts a
// `BaseOptimizerProvider` with a hand-built `IOptimizerAPI` shaped on the
// generated `EvaluateResponse` / `RotationResponse` / `OptimizeResponse`.
// The `IBoard` is derived with the same `buildBoard` port as the BaseBoard
// preview over the live `/api/base/catalog` slot table. Figures are sized to
// the capture clock (2024-05-15): a mature 2-4-3 base at roughly 46k LMD/day.

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


// ---------------------------------------------------------------------------
// The optimizer context
// ---------------------------------------------------------------------------

const noop = () => {};

const phases = (seats: number[], power: number[]) => seats.map((max_stationed, i) => ({ level: i + 1, max_stationed, electricity: power[i], manpower_cost: 0 }));

// `Catalog` = Map<FacilityType, ICatalogRoom>; the panel reads names and phases.
const CATALOG = new Map(
    (
        [
            ["CONTROL", [1, 2, 3, 4, 5], [0, 0, 0, 0, 0]],
            ["TRADING", [1, 2, 3], [-10, -30, -60]],
            ["MANUFACTURE", [1, 2, 3], [-10, -30, -60]],
            ["POWER", [1, 1, 1], [60, 130, 270]],
            ["DORMITORY", [5, 5, 5, 5, 5], [-10, -20, -30, -45, -65]],
            ["MEETING", [2, 2, 2], [-10, -30, -60]],
            ["HIRE", [1, 1, 1], [-10, -30, -60]],
            ["WORKSHOP", [1, 1, 1], [-10, -10, -10]],
            ["TRAINING", [2, 2, 2], [-10, -30, -60]],
            ["PRIVATE", [0, 0, 0], [0, 0, 0]],
            ["ELEVATOR", [0], [0]],
            ["CORRIDOR", [0], [0]],
        ] as [Facility, number[], number[]][]
    ).map(([room_type, seats, power]) => [room_type, { room_type, name: ROOMS[room_type][0], category: ROOMS[room_type][1], max_count: 1, size_col: 4, size_row: 2, phases: phases(seats, power) }]),
);

const FORMULAS = [
    { formula_type: "F_DIAMOND", label: "Originium Shard" },
    { formula_type: "F_EXP", label: "Battle Records" },
    { formula_type: "F_GOLD", label: "Pure Gold" },
];

const FORMULA_BY_SLOT: Record<string, string> = { slot_14: "F_GOLD", slot_15: "F_GOLD", slot_5: "F_EXP", slot_6: "F_EXP" };

const LAYOUT = MATURE_BASE.map((r) => ({ ...r, formula_type: FORMULA_BY_SLOT[r.slot_id] ?? null, comfort: r.room_type === "DORMITORY" ? 4000 : 0 }));

const who = (id: string, bench?: boolean) => ({ operator_id: id, name: NAMES[id] ?? id, ...(bench ? { bench } : {}) });

const scored = (slot_id: string, total_efficiency: number, yields: { lmd?: number; gold?: number; exp?: number }, extra: Record<string, unknown> = {}) => {
    const room = LAYOUT.find((r) => r.slot_id === slot_id);
    if (!room) throw new Error(slot_id);
    return {
        slot_id,
        room_type: room.room_type,
        level: room.level,
        formula_type: room.formula_type,
        total_efficiency,
        order_value: 0,
        locked: false,
        operators: room.operators.map((id) => who(id)),
        yield_lmd_per_day: yields.lmd ?? 0,
        yield_gold_per_day: yields.gold ?? 0,
        yield_exp_per_day: yields.exp ?? 0,
        non_production: [],
        ledger: [],
        ...extra,
    };
};

const ASSIGNMENT = {
    rooms: [
        scored("slot_24", 118, { lmd: 26_120 }, { capacity: 16, fill_hours: 19.6 }),
        scored("slot_25", 107, { lmd: 20_240 }, { capacity: 14, fill_hours: 21.4 }),
        scored("slot_14", 96, { gold: 63.1 }, { capacity: 66, fill_hours: 37.1 }),
        scored("slot_15", 85, { gold: 59.4 }, { capacity: 54, fill_hours: 31.8 }),
        scored("slot_5", 92, { exp: 11_480 }, { capacity: 78, fill_hours: 26.5 }),
        scored("slot_6", 88, { exp: 10_960 }, { capacity: 66, fill_hours: 23.2 }),
        scored("slot_26", 15, {}),
        scored("slot_16", 10, {}),
        scored("slot_7", 5, {}),
        scored("slot_34", 0, {}, { non_production: [{ room_type: "TRADING", value: 7 }, { room_type: "MANUFACTURE", value: 2 }] }),
        scored("slot_36", 25, {}),
        scored("slot_32", 30, {}),
    ],
    total_production_efficiency: 586,
    yield_lmd_per_day: 46_360,
    yield_exp_per_day: 22_440,
    yield_total_value: 91_780,
};

const BASELINE = {
    ...ASSIGNMENT,
    rooms: ASSIGNMENT.rooms.map((r) => (r.slot_id === "slot_14" ? { ...r, total_efficiency: 78, yield_gold_per_day: 51.2 } : r.slot_id === "slot_15" ? { ...r, total_efficiency: 70, yield_gold_per_day: 48.9 } : r)),
    total_production_efficiency: 553,
    yield_total_value: 86_400,
};

const sustain = (id: string, slot_id: string, drain: number, lasts: number | null, morale: number) => ({ operator_id: id, name: NAMES[id] ?? id, slot_id, room_type: LAYOUT.find((r) => r.slot_id === slot_id)?.room_type ?? "TRADING", drain_per_hour: drain, lasts_hours: lasts, morale });

const EVALUATION = {
    assignment: ASSIGNMENT,
    power: { generated: 810, consumed: 545, net: 265 },
    sustain: [
        sustain("char_102_texas", "slot_25", 1.3, 13.8, 18),
        sustain("char_140_whitew", "slot_25", 0.9, 24.4, 22),
        sustain("char_272_strong", "slot_24", 1.0, 9.0, 9),
        sustain("char_4032_provs", "slot_24", 1.0, 21.0, 21),
        sustain("char_190_clour", "slot_14", 0.75, 29.3, 22),
        sustain("char_336_folivo", "slot_14", 1.0, 16.0, 16),
        sustain("char_163_hpsts", "slot_14", 0.85, 25.9, 22),
        sustain("char_128_plosis", "slot_5", 1.0, 24.0, 24),
        sustain("char_2014_nian", "slot_5", 1.0, 20.0, 20),
        sustain("char_2015_dusk", "slot_5", 1.0, 11.5, 11.5),
        sustain("char_277_sqrrel", "slot_26", 0.5, null, 24),
        sustain("char_4080_lin", "slot_32", 0.75, 30.7, 23),
    ],
    dorms: {
        count: 4,
        total_levels: 12,
        total_capacity: 20,
        recovery_per_hour: 2.35,
        per_dorm: [
            { slot_id: "slot_28", level: 5, capacity: 5, recovery_per_hour: 2.65, occupant_aura_per_hour: 0.15, occupant_single_per_hour: 0, comfort: 5000, comfort_limit: 5000, comfort_upside_per_hour: 0 },
            { slot_id: "slot_20", level: 3, capacity: 5, recovery_per_hour: 2.3, occupant_aura_per_hour: 0.15, occupant_single_per_hour: 0, comfort: 2600, comfort_limit: 3000, comfort_upside_per_hour: 0.08 },
            { slot_id: "slot_9", level: 3, capacity: 5, recovery_per_hour: 2.3, occupant_aura_per_hour: 0, occupant_single_per_hour: 0, comfort: 3000, comfort_limit: 3000, comfort_upside_per_hour: 0 },
            { slot_id: "slot_3", level: 1, capacity: 5, recovery_per_hour: 1.8, occupant_aura_per_hour: 0, occupant_single_per_hour: 0, comfort: 800, comfort_limit: 1000, comfort_upside_per_hour: 0.04 },
        ],
    },
    claim: {
        next_full_hours: 19.6,
        intervals: [
            { hours: 6, lost_lmd_per_day: 0, lost_gold_per_day: 0, lost_exp_per_day: 0 },
            { hours: 12, lost_lmd_per_day: 0, lost_gold_per_day: 0, lost_exp_per_day: 0 },
            { hours: 24, lost_lmd_per_day: 4_180, lost_gold_per_day: 0, lost_exp_per_day: 620 },
        ],
    },
    unrotated: { verdict: "depletes", horizon_hours: 168, depleted: [{ operator: who("char_272_strong"), at_hours: 9.0, room_type: "TRADING" }, { operator: who("char_2015_dusk"), at_hours: 11.5, room_type: "MANUFACTURE" }, { operator: who("char_102_texas"), at_hours: 13.8, room_type: "TRADING" }], dorm_overflow: 0, timeline: [] },
    morale_synced_hours_ago: 2.4,
    trainer_hints: [],
    drones: { current: 88, max: 145, full_in_hours: 15.3 },
};

const facility = (slot_id: string, lmd: number, gold: number, exp: number, idle_hours: number) => ({ slot_id, room_type: LAYOUT.find((r) => r.slot_id === slot_id)?.room_type ?? "TRADING", formula_type: FORMULA_BY_SLOT[slot_id] ?? null, lmd, gold, exp, idle_hours });

const shiftRoom = (slot_id: string, recommended: string[], swapIn: string[], swapOut: string[], efficiency: number) => ({
    slot_id,
    room_type: LAYOUT.find((r) => r.slot_id === slot_id)?.room_type ?? "TRADING",
    formula_type: FORMULA_BY_SLOT[slot_id] ?? null,
    active: true,
    recommended: recommended.map((id) => who(id)),
    current: LAYOUT.find((r) => r.slot_id === slot_id)?.operators.map((id) => who(id)) ?? [],
    swap_in: swapIn.map((id) => who(id)),
    swap_out: swapOut.map((id) => who(id)),
    moved_out: [],
    matches: swapIn.length === 0,
    equivalent: swapIn.length === 0,
    gap_pct: null,
    efficiency,
    ledger: [],
    team_id: null,
    team_label: null,
});

const ROTATION = {
    shift_count: 3,
    rotation: {
        shifts: [
            { index: 1, rooms: [shiftRoom("slot_24", ["char_272_strong", "char_4032_provs", "char_486_takila"], [], [], 118), shiftRoom("slot_25", ["char_254_vodfox", "char_102_texas", "char_140_whitew"], [], [], 107)] },
            { index: 2, rooms: [shiftRoom("slot_24", ["char_272_strong", "char_4032_provs", "char_486_takila"], [], [], 118), shiftRoom("slot_25", ["char_254_vodfox", "char_102_texas", "char_103_angel"], ["char_103_angel"], ["char_140_whitew"], 101)] },
            { index: 3, rooms: [shiftRoom("slot_24", ["char_272_strong", "char_4032_provs", "char_486_takila"], [], [], 118), shiftRoom("slot_25", ["char_254_vodfox", "char_102_texas", "char_140_whitew"], ["char_140_whitew"], ["char_103_angel"], 107)] },
        ],
        sustained: [who("char_272_strong")],
        sustainability: {
            verdict: "holds_up",
            horizon_hours: 168,
            depleted: [],
            dorm_overflow: 0,
            timeline: [],
            facilities: [facility("slot_24", 182_840, 0, 0, 0), facility("slot_25", 141_680, 0, 0, 0), facility("slot_14", 0, 441.7, 0, 0), facility("slot_15", 0, 415.8, 0, 0), facility("slot_5", 0, 0, 80_360, 0), facility("slot_6", 0, 0, 76_720, 1.4)],
        },
    },
};

const PROPOSAL = {
    proposal: ASSIGNMENT,
    baseline: BASELINE,
    room_diffs: [
        { slot_id: "slot_14", room_type: "MANUFACTURE", before: ["char_190_clour", "char_336_folivo", "char_400_weedy"], after: ["char_190_clour", "char_336_folivo", "char_163_hpsts"], efficiency_before: 78, efficiency_after: 96, yield_before: 51.2, yield_after: 63.1 },
        { slot_id: "slot_15", room_type: "MANUFACTURE", before: ["char_163_hpsts", "char_196_sunbr", "char_4105_almond"], after: ["char_400_weedy", "char_196_sunbr", "char_4105_almond"], efficiency_before: 70, efficiency_after: 85, yield_before: 48.9, yield_after: 59.4 },
    ],
    power: EVALUATION.power,
};

function api(overrides: Record<string, unknown> = {}) {
    return {
        layout: LAYOUT,
        dirty: false,
        catalog: CATALOG,
        slots: [],
        formulas: FORMULAS,
        presets: [],
        shiftCount: 3,
        boardRooms: LAYOUT,
        catalogLoading: false,
        layoutLoading: false,
        evaluation: EVALUATION,
        evaluating: false,
        evaluationError: null,
        rotation: ROTATION,
        rotationLoading: false,
        ignorePromotion: false,
        setIgnorePromotion: noop,
        openRecruitSlots: 2,
        setOpenRecruitSlots: noop,
        factsSaved: null,
        trainingClass: null,
        setTrainingClass: noop,
        claimIntervalHours: undefined,
        setClaimIntervalHours: noop,
        viewShift: null,
        setViewShift: noop,
        shiftRoom: () => undefined,
        proposal: null,
        optimizing: false,
        optimizeError: null,
        runOptimize: noop,
        reset: noop,
        ...overrides,
    };
}

const BOARD = buildBoard(MATURE_BASE, NAMES);

/** The scored draft before any optimizer run: headline figures, the plain Optimize action, Deep Dive open on check-in economics and per-facility output. */
export const ScoredDraft = () => (
    <BaseOptimizerProvider value={api()}>
        <OptimizerBasePanel board={BOARD} />
    </BaseOptimizerProvider>
);

/** After Optimize: Reset and Re-optimize actions, the "Holds up" sustainability verdict, simulated totals under the rotation. */
export const Planned = () => (
    <BaseOptimizerProvider value={api({ proposal: PROPOSAL, dirty: true })}>
        <OptimizerBasePanel board={BOARD} />
    </BaseOptimizerProvider>
);

/** Still scoring: every headline a dash, the rotation simulating, no Deep Dive yet. */
export const Scoring = () => (
    <BaseOptimizerProvider value={api({ evaluation: undefined, evaluating: true, rotation: null, rotationLoading: true })}>
        <OptimizerBasePanel board={BOARD} />
    </BaseOptimizerProvider>
);

/** The backend refused the draft: both error strips above the board, the optimizer action disabled while a run is in flight. */
export const ScoringFailed = () => (
    <BaseOptimizerProvider value={api({ evaluation: undefined, rotation: null, evaluationError: new Error("Operator char_4032_provs is seated in two rooms."), optimizeError: new Error("Couldn't reach the server. Check your connection and try again."), optimizing: true })}>
        <OptimizerBasePanel board={BOARD} />
    </BaseOptimizerProvider>
);
