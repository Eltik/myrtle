import { BaseOptimizerProvider, RoomTile } from "frontend";
import { type ReactNode, useEffect } from "react";

// RoomPopover is the panel a board tile opens: the room's stats, every seated
// operator with their RIIC kit, and beside each skill the ledger chip saying
// what that line is worth in THIS crew (a marginal from the Rust clause
// engine). It reads the optimizer context, so each story mounts a
// `BaseOptimizerProvider` with a hand-built `IOptimizerAPI` and renders the
// real `RoomTile`, then clicks it two frames after mount - the popover is the
// tile's own, anchored under it, exactly as on the board. Kits are the live
// `/api/operators/<id>` data; ids checked against `/api/operators/index`.

// ---------------------------------------------------------------------------
// Catalog subset (live `/api/base/catalog`)
// ---------------------------------------------------------------------------

const phases = (seats: number[], power: number[]) => seats.map((max_stationed, i) => ({ level: i + 1, max_stationed, electricity: power[i], manpower_cost: 0 }));

const CATALOG = new Map<string, { room_type: string; name: string; category: string; max_count: number; size_col: number; size_row: number; phases: ReturnType<typeof phases> }>([
    ["TRADING", { room_type: "TRADING", name: "Trading Post", category: "OUTPUT", max_count: 5, size_col: 4, size_row: 2, phases: phases([1, 2, 3], [-10, -30, -60]) }],
    ["MANUFACTURE", { room_type: "MANUFACTURE", name: "Factory", category: "OUTPUT", max_count: 5, size_col: 4, size_row: 2, phases: phases([1, 2, 3], [-10, -30, -60]) }],
    ["POWER", { room_type: "POWER", name: "Power Plant", category: "OUTPUT", max_count: 3, size_col: 4, size_row: 2, phases: phases([1, 1, 1], [60, 130, 270]) }],
    ["MEETING", { room_type: "MEETING", name: "Reception Room", category: "FUNCTION", max_count: 1, size_col: 8, size_row: 2, phases: phases([2, 2, 2], [-10, -30, -60]) }],
    ["DORMITORY", { room_type: "DORMITORY", name: "Dormitory", category: "CUSTOM", max_count: 4, size_col: 6, size_row: 2, phases: phases([5, 5, 5, 5, 5], [-10, -20, -30, -45, -65]) }],
]);

const FORMULAS = [
    { formula_type: "F_DIAMOND", label: "Originium Shard" },
    { formula_type: "F_EXP", label: "Battle Records" },
    { formula_type: "F_GOLD", label: "Pure Gold" },
];

// ---------------------------------------------------------------------------
// Kits
// ---------------------------------------------------------------------------

type Skill = { buffId: string; buffName: string; description: string; roomType: string; efficiency: number; targets: string[]; skillIcon: string; unlockElite: number; unlockLevel: number; slot: number; unlocked: boolean; live: boolean };

const sk = (buffId: string, buffName: string, description: string, roomType: string, skillIcon: string, unlockElite: number, slot: number, flags: Partial<Pick<Skill, "unlocked" | "live" | "efficiency" | "targets">> = {}): Skill => ({
    buffId,
    buffName,
    description,
    roomType,
    efficiency: 0,
    targets: [],
    skillIcon,
    unlockElite,
    unlockLevel: 1,
    slot,
    unlocked: true,
    live: true,
    ...flags,
});

const TEXAS = [
    sk("trade_ord_spd&cost_P[000]", "Feud", "When this Operator is assigned to the same Trading Post as <@cc.kw>Lappland</>, Morale consumed each hour <@cc.vdown>+0.3</>, and order acquisition efficiency <@cc.vup>+65%</>", "TRADING", "bskill_tra_texas1", 0, 0),
    sk("trade_ord_limit&cost_P[010]", "Tacit Understanding", "When this Operator is assigned to the same Trading Post as <@cc.kw>Exusiai</>, Morale consumed each hour <@cc.vup>-0.3</>", "TRADING", "bskill_tra_texas2", 2, 1),
];
const LAPPLAND = [
    sk("trade_ord_limit&cost_P[000]", "Hidden Purpose α", "When this Operator is assigned to the same Trading Post as <@cc.kw>Texas</>, Morale consumed each hour <@cc.vup>-0.1</>, and order limit <@cc.vup>+2</>", "TRADING", "bskill_tra_Lappland1", 0, 0, { live: false }),
    sk("trade_ord_limit&cost_P[001]", "Hidden Purpose β", "When this Operator is assigned to the same Trading Post as <@cc.kw>Texas</>, Morale consumed each hour <@cc.vup>-0.1</>, and order limit <@cc.vup>+4</>", "TRADING", "bskill_tra_Lappland2", 2, 0),
];
const EXUSIAI = [
    sk("trade_ord_spd[010]", "Penguin Logistics α", "When this Operator is assigned to a Trading Post, order acquisition efficiency <@cc.vup>+20%</>", "TRADING", "bskill_tra_spd1", 0, 0, { efficiency: 20, live: false }),
    sk("trade_ord_spd[020]", "Logistics Expert", "When this Operator is assigned to a Trading Post, order acquisition efficiency <@cc.vup>+35%</>", "TRADING", "bskill_tra_spd3", 2, 0, { efficiency: 35 }),
];
const VERMEIL = [
    sk("manu_prod_limit&cost[0000]", "Junkman", "When this Operator is assigned to a Factory, capacity limit <@cc.vup>+8</> and Morale consumed per hour <@cc.vup>-0.25</>", "MANUFACTURE", "bskill_man_limit&cost1", 0, 0),
    sk("manu_prod_spd_variable[000]", "Recycling", "When this Operator is assigned to a Factory, Operators assigned to the Factory increase all capacity limits, add <@cc.vup>2%</> productivity.", "MANUFACTURE", "bskill_man_spd_variable11", 1, 1, { targets: ["F_GOLD", "F_EXP", "F_DIAMOND"] }),
];
const SCENE = [
    sk("manu_prod_spd_addition[041]", "Time-Lapse Photography", "When this Operator is assigned to a Factory, productivity <@cc.vup>+15%</> in the first hour and thereafter <@cc.vup>+2%</> per hour, up to <@cc.vup>+25%</>", "MANUFACTURE", "bskill_man_spd_add2", 0, 0, { efficiency: 15, targets: ["F_GOLD", "F_EXP", "F_DIAMOND"] }),
    sk("manu_formula_limit[0000]", "Editing α", "When this Operator is assigned to a Factory, capacity limit is increased by <@cc.vup>+12</> when producing <@cc.kw>Battle Records</>", "MANUFACTURE", "bskill_man_exp&limit1", 2, 1, { targets: ["F_EXP"] }),
];
const VULCAN = [
    sk("manu_prod_spd&limit&cost[000]", "Craftsmanship Spirit α", "When this Operator is assigned to a Factory, productivity <@cc.vdown>-5%</>, capacity limit <@cc.vup>+16</>, and Morale consumed per hour <@cc.vup>-0.15</>", "MANUFACTURE", "bskill_man_spd&limit&cost1", 0, 0, { efficiency: -5, targets: ["F_GOLD", "F_EXP", "F_DIAMOND"] }),
    sk("manu_prod_spd&limit&cost[001]", "Craftsmanship Spirit β", "When this Operator is assigned to a Factory, productivity <@cc.vdown>-5%</>, capacity limit <@cc.vup>+19</>, and Morale consumed per hour <@cc.vup>-0.25</>", "MANUFACTURE", "bskill_man_spd&limit&cost2", 2, 0, { efficiency: -5, targets: ["F_GOLD", "F_EXP", "F_DIAMOND"], unlocked: false, live: false }),
];
const SORA = [sk("trade_ord_spd[011]", "Penguin Logistics β", "When this Operator is assigned to a Trading Post, order acquisition efficiency <@cc.vup>+30%</>", "TRADING", "bskill_tra_spd2", 2, 1, { efficiency: 30 })];
const GITANO = [sk("meet_spd[030]", "Divination", "When this Operator is assigned to the Reception Room, Clue search speed increases by <@cc.vup>25%</>", "MEETING", "bskill_meet_spd3", 1, 1, { efficiency: 25 })];

// ---------------------------------------------------------------------------
// Tiles (an `ITile` as `buildBoard` shapes it; placed at grid origin)
// ---------------------------------------------------------------------------

const op = (id: string, name: string, skills: Skill[], roomType: string) => ({ id, name, skills: skills.filter((s) => s.roomType === roomType) });

const tile = (slotId: string, facility: string, name: string, level: number, maxPhase: number, seats: number, w: number, operators: ReturnType<typeof op>[]) => ({
    slotId,
    kind: "flexible" as const,
    facility,
    name,
    level,
    maxPhase,
    built: true,
    operators,
    seats,
    col: 1,
    row: 1,
    w,
    h: 1,
});

const TRADING_TILE = tile("slot_25", "TRADING", "Trading Post", 3, 3, 3, 2, [op("char_102_texas", "Texas", TEXAS, "TRADING"), op("char_140_whitew", "Lappland", LAPPLAND, "TRADING"), op("char_103_angel", "Exusiai", EXUSIAI, "TRADING")]);
const FACTORY_TILE = tile("slot_14", "MANUFACTURE", "Factory", 3, 3, 3, 2, [op("char_190_clour", "Vermeil", VERMEIL, "MANUFACTURE"), op("char_336_folivo", "Scene", SCENE, "MANUFACTURE"), op("char_163_hpsts", "Vulcan", VULCAN, "MANUFACTURE")]);
const MEETING_TILE = { ...tile("slot_36", "MEETING", "Reception Room", 3, 3, 2, 4, [op("char_101_sora", "Sora", SORA, "MEETING"), op("char_109_fmout", "Gitano", GITANO, "MEETING")]), kind: "fixed" as const };
const EMPTY_TRADING_TILE = tile("slot_24", "TRADING", "Trading Post", 2, 3, 2, 2, []);

// ---------------------------------------------------------------------------
// The optimizer context
// ---------------------------------------------------------------------------

const noop = () => {};

const draft = (slot_id: string, room_type: string, level: number, operators: string[], formula_type: string | null = null) => ({ slot_id, room_type, level, operators, formula_type, comfort: 0 });

const BOARD_ROOMS = [
    draft("slot_25", "TRADING", 3, ["char_102_texas", "char_140_whitew", "char_103_angel"]),
    draft("slot_14", "MANUFACTURE", 3, ["char_190_clour", "char_336_folivo", "char_163_hpsts"], "F_GOLD"),
    draft("slot_36", "MEETING", 3, ["char_101_sora", "char_109_fmout"]),
    draft("slot_24", "TRADING", 2, []),
];

const line = (operator_id: string, operator_name: string, buff_id: string, buff_name: string, speed_pct: number, disposition: string, extra: Record<string, unknown> = {}) => ({ operator_id, operator_name, buff_id, buff_name, speed_pct, disposition, ...extra });

const scoredRoom = (slot_id: string, room_type: string, level: number, formula_type: string | null, total_efficiency: number, operators: { operator_id: string; name: string; bench?: boolean }[], yields: { lmd?: number; gold?: number; exp?: number }, ledger: ReturnType<typeof line>[], extra: Record<string, unknown> = {}) => ({
    slot_id,
    room_type,
    level,
    formula_type,
    total_efficiency,
    order_value: 0,
    locked: false,
    operators,
    yield_lmd_per_day: yields.lmd ?? 0,
    yield_gold_per_day: yields.gold ?? 0,
    yield_exp_per_day: yields.exp ?? 0,
    non_production: [],
    ledger,
    ...extra,
});

const TRADING_LEDGER = [
    line("char_102_texas", "Texas", "trade_ord_spd&cost_P[000]", "Feud", 65, "contributes"),
    line("char_102_texas", "Texas", "trade_ord_limit&cost_P[010]", "Tacit Understanding", 0, "morale"),
    line("char_140_whitew", "Lappland", "trade_ord_limit&cost_P[001]", "Hidden Purpose β", 0, "capacity"),
    line("char_103_angel", "Exusiai", "trade_ord_spd[020]", "Logistics Expert", 35, "contributes"),
    line("char_002_amiya", "Amiya", "control_tra_spd[000]", "Agreement", 7, "contributes", { from_control_center: true }),
];

const FACTORY_LEDGER = [
    line("char_190_clour", "Vermeil", "manu_prod_limit&cost[0000]", "Junkman", 0, "capacity"),
    line("char_190_clour", "Vermeil", "manu_prod_spd_variable[000]", "Recycling", 6, "contributes", { note: "Counts the three operators seated here, itself included - the 6% is spread over what it counts." }),
    line("char_336_folivo", "Scene", "manu_prod_spd_addition[041]", "Time-Lapse Photography", 25, "contributes"),
    line("char_336_folivo", "Scene", "manu_formula_limit[0000]", "Editing α", 0, "inactive"),
    line("char_163_hpsts", "Vulcan", "manu_prod_spd&limit&cost[000]", "Craftsmanship Spirit α", -5, "contributes"),
    line("char_003_kalts", "Kal'tsit", "control_prod_spd[000]", "Highest Authority", 2, "contributes", { from_control_center: true }),
];

const MEETING_LEDGER = [line("char_109_fmout", "Gitano", "meet_spd[030]", "Divination", 25, "non_production")];

const EVALUATION = {
    assignment: {
        rooms: [
            scoredRoom("slot_25", "TRADING", 3, null, 107, [{ operator_id: "char_102_texas", name: "Texas" }, { operator_id: "char_140_whitew", name: "Lappland" }, { operator_id: "char_103_angel", name: "Exusiai" }], { lmd: 24_480 }, TRADING_LEDGER, { capacity: 14, fill_hours: 21.4 }),
            scoredRoom("slot_14", "MANUFACTURE", 3, "F_GOLD", 78, [{ operator_id: "char_190_clour", name: "Vermeil" }, { operator_id: "char_336_folivo", name: "Scene" }, { operator_id: "char_163_hpsts", name: "Vulcan" }], { gold: 51.2 }, FACTORY_LEDGER, { capacity: 66, fill_hours: 37.1 }),
            scoredRoom("slot_36", "MEETING", 3, null, 25, [{ operator_id: "char_101_sora", name: "Sora" }, { operator_id: "char_109_fmout", name: "Gitano" }], {}, MEETING_LEDGER),
        ],
        total_production_efficiency: 185,
        yield_lmd_per_day: 24_480,
        yield_exp_per_day: 0,
        yield_total_value: 42_900,
    },
    power: { generated: 810, consumed: 505, net: 305 },
    sustain: [],
    dorms: { count: 4, total_levels: 12, total_capacity: 20, recovery_per_hour: 2.35, per_dorm: [] },
    claim: null,
    unrotated: null,
    trainer_hints: [],
    drones: null,
};

// After "Optimize": the factory swapped Weedy out for Vulcan, who sits on a
// spare seat for his capacity; the trading post's crew was already optimal.
const PROPOSAL = {
    proposal: {
        ...EVALUATION.assignment,
        rooms: EVALUATION.assignment.rooms.map((r) => (r.slot_id === "slot_14" ? { ...r, total_efficiency: 96, operators: [{ operator_id: "char_190_clour", name: "Vermeil" }, { operator_id: "char_336_folivo", name: "Scene" }, { operator_id: "char_163_hpsts", name: "Vulcan", bench: true }] } : r)),
        total_production_efficiency: 203,
    },
    baseline: EVALUATION.assignment,
    room_diffs: [{ slot_id: "slot_14", room_type: "MANUFACTURE", before: ["char_190_clour", "char_336_folivo", "char_400_weedy"], after: ["char_190_clour", "char_336_folivo", "char_163_hpsts"], efficiency_before: 78, efficiency_after: 96, yield_before: 51.2, yield_after: 63.1 }],
    power: EVALUATION.power,
};

const ROTATION = {
    shift_count: 3,
    rotation: {
        shifts: [],
        sustained: [{ operator_id: "char_190_clour", name: "Vermeil" }],
        sustainability: { verdict: "holds_up", horizon_hours: 168, depleted: [], dorm_overflow: 0, timeline: [] },
    },
};

function api(overrides: Record<string, unknown> = {}) {
    return {
        layout: BOARD_ROOMS,
        dirty: false,
        catalog: CATALOG,
        slots: [],
        formulas: FORMULAS,
        presets: [],
        shiftCount: 3,
        boardRooms: BOARD_ROOMS,
        catalogLoading: false,
        layoutLoading: false,
        evaluation: EVALUATION,
        evaluating: false,
        evaluationError: null,
        rotation: null,
        rotationLoading: false,
        ignorePromotion: false,
        setIgnorePromotion: noop,
        openRecruitSlots: 0,
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

// ---------------------------------------------------------------------------
// Stage: one real tile on a scrap of the board, clicked open two frames after
// mount (an effect-time click is dropped), then blurred so the brand-red focus
// ring does not read as part of the popover. `.ds-single` makes the story root
// the fixed-position container, so the stage owns the height the popup needs.
// ---------------------------------------------------------------------------

function Stage({ value, units, children }: { value: ReturnType<typeof api>; units: number; children: ReactNode }) {
    useEffect(() => {
        let f2 = 0;
        let f3 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                document.querySelector<HTMLButtonElement>("button[data-slot-id]")?.click();
                f3 = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur());
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            cancelAnimationFrame(f3);
        };
    }, []);
    return (
        <BaseOptimizerProvider value={value}>
            <div className="min-h-[520px] rounded-xl border border-border bg-card p-3">
                <div className="flex justify-center rounded-lg px-4 py-5" style={{ backgroundColor: "#191919" }}>
                    <div style={{ display: "grid", gridTemplateColumns: `repeat(${units}, 64px)`, gridAutoRows: "64px", gap: 3 }}>{children}</div>
                </div>
            </div>
        </BaseOptimizerProvider>
    );
}

/** A scored trading post: kits with marginal chips, a morale-only line, a capacity line and the Control Center's credit. */
export const TradingPostScored = () => (
    <Stage units={2} value={api()}>
        <RoomTile tile={TRADING_TILE} />
    </Stage>
);

/** After Optimize: the room diff footer, a 24/7 pick, a benched spare seat, a locked tier, an inactive line and a spread-count note. */
export const FactoryOptimized = () => (
    <Stage units={2} value={api({ proposal: PROPOSAL, rotation: ROTATION })}>
        <RoomTile tile={FACTORY_TILE} />
    </Stage>
);

/** A non-producing room reports its figure in its own units - Clue search - with the reception-only ledger label. */
export const ReceptionRoom = () => (
    <Stage units={4} value={api()}>
        <RoomTile tile={MEETING_TILE} />
    </Stage>
);

/** An unstaffed room while the draft is still being scored. */
export const UnstaffedScoring = () => (
    <Stage units={2} value={api({ evaluation: undefined, evaluating: true })}>
        <RoomTile tile={EMPTY_TRADING_TILE} />
    </Stage>
);
