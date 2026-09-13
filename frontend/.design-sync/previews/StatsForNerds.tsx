import { BaseOptimizerProvider, StatsForNerds } from "frontend";
import { useEffect, useRef } from "react";
import type { IEvaluateResponse, IOptimizeResponse, IRotationResponse } from "../../src/lib/api/base";
import type { IMoraleTimeline } from "../../src/lib/api/user";
import type { Catalog } from "../../src/lib/base/catalog";
import type { IOptimizerAPI } from "../../src/lib/base/use-optimizer";

// StatsForNerds takes no props: it reads the base optimizer's context
// (`useBaseOptimizer()`) and lays the raw numbers out in a two-column grid -
// output, power, dormitories and crew from `evaluation`; "optimized vs your
// base" and per-room changes once `proposal` exists; the rotation verdict and
// the morale-over-time chart once `rotation` exists. It renders nothing until
// `evaluation` exists, and is COLLAPSED by default - BasePanel mounts it last,
// under the Deep Dive.
//
// The fixture is one coherent 2-4-3 base (2 trading posts, 4 factories, 3 power
// plants, 4 dorms) on the yield model the engine uses: a factory makes
// 20 × (1 + eff%) gold bars/day, a post sells the same at 500 LMD a bar, an EXP
// factory makes 8,000 × (1 + eff%) EXP/day, and the headline LMD is the gold
// the posts can sell - so the two posts' figures sum to the headline.

const noop = () => undefined;

const op = (operator_id: string, name: string) => ({ operator_id, name });

const phases = (stationed: number, power: number[]) => power.map((electricity, i) => ({ level: i + 1, max_stationed: stationed, electricity, manpower_cost: 100 }));

const CATALOG: Catalog = new Map([
    ["CONTROL", { room_type: "CONTROL", name: "Control Center", category: "SPECIAL", max_count: 1, size_col: 4, size_row: 1, phases: phases(5, [0, 0, 0, 0, 0]) }],
    ["TRADING", { room_type: "TRADING", name: "Trading Post", category: "OUTPUT", max_count: 5, size_col: 3, size_row: 1, phases: phases(3, [-10, -30, -60]) }],
    ["MANUFACTURE", { room_type: "MANUFACTURE", name: "Factory", category: "OUTPUT", max_count: 5, size_col: 3, size_row: 1, phases: phases(3, [-10, -30, -60]) }],
    ["POWER", { room_type: "POWER", name: "Power Plant", category: "OUTPUT", max_count: 3, size_col: 3, size_row: 1, phases: phases(1, [60, 130, 270]) }],
    ["DORMITORY", { room_type: "DORMITORY", name: "Dormitory", category: "CUSTOM", max_count: 4, size_col: 5, size_row: 1, phases: phases(5, [-10, -20, -30, -45, -65]) }],
    ["MEETING", { room_type: "MEETING", name: "Reception Room", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(2, [-10, -30, -60]) }],
    ["HIRE", { room_type: "HIRE", name: "Office", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(1, [-10, -30, -60]) }],
    ["TRAINING", { room_type: "TRAINING", name: "Training Room", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(2, [-10, -30, -60]) }],
    ["WORKSHOP", { room_type: "WORKSHOP", name: "Workshop", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(1, [-10, -10, -10]) }],
]);

// Crews of the drafted base.
const TEXAS = op("char_102_texas", "Texas");
const LAPPLAND = op("char_140_whitew", "Lappland");
const EXUSIAI = op("char_103_angel", "Exusiai");
const FIAMMETTA = op("char_300_phenxi", "Fiammetta");
const SHAMARE = op("char_254_vodfox", "Shamare");
const JAYE = op("char_272_strong", "Jaye");
const BISON = op("char_325_bison", "Bison");
const VERMEIL = op("char_190_clour", "Vermeil");
const SCAVENGER = op("char_149_scave", "Scavenger");
const STEWARD = op("char_210_stward", "Steward");
const GUMMY = op("char_196_sunbr", "Gummy");
const CEOBE = op("char_2013_cerber", "Ceobe");
const CUORA = op("char_150_snakek", "Cuora");
const PERFUMER = op("char_181_flower", "Perfumer");
const PTILOPSIS = op("char_128_plosis", "Ptilopsis");
const VULCAN = op("char_163_hpsts", "Vulcan");
const WEEDY = op("char_400_weedy", "Weedy");
const DOROTHY = op("char_4048_doroth", "Dorothy");
const SILENCE = op("char_108_silent", "Silence");
const GREYY = op("char_253_greyy", "Greyy");
const SHAW = op("char_277_sqrrel", "Shaw");
const SPOT = op("char_284_spot", "Spot");
const AMIYA = op("char_002_amiya", "Amiya");
const ROSMONTIS = op("char_391_rosmon", "Rosmontis");
const WHISPERAIN = op("char_436_whispr", "Whisperain");
const DUSK = op("char_2015_dusk", "Dusk");
const LING = op("char_2023_ling", "Ling");
const ELYSIUM = op("char_401_elysm", "Elysium");
const GNOSIS = op("char_206_gnosis", "Gnosis");
const ORCHID = op("char_278_orchid", "Orchid");
const ACIDDROP = op("char_366_acdrop", "Aciddrop");
const FIREWATCH = op("char_158_milu", "Firewatch");
const NIAN = op("char_2014_nian", "Nian");

type Room = IEvaluateResponse["assignment"]["rooms"][number];

const room = (slot_id: string, room_type: string, level: number, formula_type: string | null, total_efficiency: number, operators: Room["operators"], extra: Partial<Room> = {}): Room => ({
    slot_id,
    room_type,
    level,
    formula_type,
    total_efficiency,
    order_value: 0,
    locked: false,
    operators,
    yield_lmd_per_day: 0,
    yield_gold_per_day: 0,
    yield_exp_per_day: 0,
    non_production: [],
    ledger: [],
    ...extra,
});

/** Trading posts sell 20 × (1 + eff) bars/day at 500 LMD; a Lv3 post buffers 10 orders (+2 with Bison). */
const TRADING_ROOMS = [
    room("slot_5", "TRADING", 3, null, 100, [TEXAS, LAPPLAND, EXUSIAI], { yield_lmd_per_day: 20_000, capacity: 10, fill_hours: 10.4 }),
    room("slot_6", "TRADING", 3, null, 88, [FIAMMETTA, SHAMARE, JAYE], { yield_lmd_per_day: 18_800, capacity: 12, fill_hours: 13.2 }),
];
/** Factories: 20 × (1 + eff) gold bars/day or 8,000 × (1 + eff) EXP/day into a 54-item Lv3 buffer. */
const FACTORY_ROOMS = [
    room("slot_7", "MANUFACTURE", 3, "F_GOLD", 95, [VERMEIL, SCAVENGER, STEWARD], { yield_gold_per_day: 39, capacity: 54, fill_hours: 33.2 }),
    room("slot_14", "MANUFACTURE", 3, "F_GOLD", 100, [GUMMY, CEOBE, CUORA], { yield_gold_per_day: 40, capacity: 54, fill_hours: 32.4 }),
    room("slot_15", "MANUFACTURE", 3, "F_EXP", 92, [PERFUMER, PTILOPSIS, VULCAN], { yield_exp_per_day: 15_360, capacity: 54, fill_hours: 84.4 }),
    room("slot_16", "MANUFACTURE", 3, "F_EXP", 78, [WEEDY, DOROTHY, SILENCE], { yield_exp_per_day: 14_240, capacity: 54, fill_hours: 91 }),
];
const OTHER_ROOMS = [
    room("slot_24", "POWER", 3, null, 15, [GREYY]),
    room("slot_25", "POWER", 3, null, 10, [SHAW]),
    room("slot_26", "POWER", 3, null, 10, [SPOT]),
    room("slot_34", "CONTROL", 5, null, 0, [AMIYA, ROSMONTIS, WHISPERAIN, DUSK, LING], { non_production: [{ room_type: "MEETING", value: 10 }] }),
    room("slot_13", "MEETING", 3, null, 0, [ELYSIUM, GNOSIS]),
    room("slot_23", "HIRE", 3, null, 0, [ORCHID]),
    room("slot_32", "TRAINING", 3, null, 0, [ACIDDROP]),
    room("slot_36", "WORKSHOP", 3, null, 0, [NIAN]),
];

type Sustain = IEvaluateResponse["sustain"][number];
const sustain = (o: { operator_id: string; name: string }, slot_id: string, room_type: string, drain_per_hour: number, morale: number): Sustain => ({
    operator_id: o.operator_id,
    name: o.name,
    slot_id,
    room_type,
    drain_per_hour,
    lasts_hours: drain_per_hour > 0 ? Math.round((morale / drain_per_hour) * 10) / 10 : null,
    morale,
});

/** Every stationed operator's endurance from their bar as of the last sync. */
const SUSTAIN: Sustain[] = [
    sustain(TEXAS, "slot_5", "TRADING", 1, 18.5),
    sustain(LAPPLAND, "slot_5", "TRADING", 1, 15),
    sustain(EXUSIAI, "slot_5", "TRADING", 1, 10.5),
    sustain(FIAMMETTA, "slot_6", "TRADING", 0, 24),
    sustain(SHAMARE, "slot_6", "TRADING", 1, 21),
    sustain(JAYE, "slot_6", "TRADING", 1, 16.4),
    sustain(VERMEIL, "slot_7", "MANUFACTURE", 1.25, 9),
    sustain(SCAVENGER, "slot_7", "MANUFACTURE", 1, 19.2),
    sustain(STEWARD, "slot_7", "MANUFACTURE", 1, 22),
    sustain(GUMMY, "slot_14", "MANUFACTURE", 1, 14.8),
    sustain(CEOBE, "slot_14", "MANUFACTURE", 1, 17.3),
    sustain(CUORA, "slot_14", "MANUFACTURE", 1, 23),
    sustain(PERFUMER, "slot_15", "MANUFACTURE", 1, 13),
    sustain(PTILOPSIS, "slot_15", "MANUFACTURE", 1, 20.1),
    sustain(VULCAN, "slot_15", "MANUFACTURE", 1, 24),
    sustain(WEEDY, "slot_16", "MANUFACTURE", 1.25, 20),
    sustain(DOROTHY, "slot_16", "MANUFACTURE", 1, 22.6),
    sustain(SILENCE, "slot_16", "MANUFACTURE", 1, 24),
    sustain(GREYY, "slot_24", "POWER", 1, 21),
    sustain(SHAW, "slot_25", "POWER", 1, 19.5),
    sustain(SPOT, "slot_26", "POWER", 1, 24),
    sustain(AMIYA, "slot_34", "CONTROL", 0.05, 22.4),
    sustain(ROSMONTIS, "slot_34", "CONTROL", 0.05, 22.4),
    sustain(WHISPERAIN, "slot_34", "CONTROL", 0.05, 22.4),
    sustain(DUSK, "slot_34", "CONTROL", 0.05, 22.4),
    sustain(LING, "slot_34", "CONTROL", 0.05, 24),
    sustain(ELYSIUM, "slot_13", "MEETING", 1, 23),
    sustain(GNOSIS, "slot_13", "MEETING", 1, 23),
    sustain(ORCHID, "slot_23", "HIRE", 1, 24),
    sustain(ACIDDROP, "slot_32", "TRAINING", 1, 24),
    sustain(NIAN, "slot_36", "WORKSHOP", 1, 24),
];

/** Who runs dry, in order, if nobody is ever swapped (from their real current bar). */
const UNROTATED_DEPLETED = SUSTAIN.filter((e) => e.lasts_hours !== null && e.lasts_hours < 20)
    .sort((a, b) => (a.lasts_hours ?? 0) - (b.lasts_hours ?? 0))
    .map((e) => ({ operator: op(e.operator_id, e.name), at_hours: e.lasts_hours ?? 0, room_type: e.room_type }));

/** The drafted base, scored: 38,800 LMD/day (77.6 bars sold of 79 made) and 29,600 EXP/day. */
const EVALUATION: IEvaluateResponse = {
    assignment: {
        rooms: [...TRADING_ROOMS, ...FACTORY_ROOMS, ...OTHER_ROOMS],
        total_production_efficiency: 588,
        yield_lmd_per_day: 38_800,
        yield_exp_per_day: 29_600,
        yield_total_value: 68_400,
    },
    power: { generated: 810, consumed: 790, net: 20 },
    sustain: SUSTAIN,
    dorms: {
        count: 4,
        total_levels: 19,
        total_capacity: 20,
        recovery_per_hour: 3.62,
        per_dorm: [
            { slot_id: "slot_3", level: 5, capacity: 5, recovery_per_hour: 2, occupant_aura_per_hour: 0, occupant_single_per_hour: 0, comfort: 5000, comfort_limit: 5000, comfort_upside_per_hour: 0 },
            { slot_id: "slot_9", level: 5, capacity: 5, recovery_per_hour: 2, occupant_aura_per_hour: 0.3, occupant_single_per_hour: 0, comfort: 5000, comfort_limit: 5000, comfort_upside_per_hour: 0 },
            { slot_id: "slot_20", level: 5, capacity: 5, recovery_per_hour: 2, occupant_aura_per_hour: 0, occupant_single_per_hour: 0.7, comfort: 4200, comfort_limit: 5000, comfort_upside_per_hour: 0.32 },
            { slot_id: "slot_28", level: 4, capacity: 5, recovery_per_hour: 1.9, occupant_aura_per_hour: 0, occupant_single_per_hour: 0, comfort: 2750, comfort_limit: 4000, comfort_upside_per_hour: 0.5 },
        ],
    },
    claim: {
        next_full_hours: 10.4,
        intervals: [
            { hours: 6, lost_lmd_per_day: 0, lost_gold_per_day: 0, lost_exp_per_day: 0 },
            { hours: 12, lost_lmd_per_day: 1_333, lost_gold_per_day: 0, lost_exp_per_day: 0 },
            { hours: 24, lost_lmd_per_day: 19_793, lost_gold_per_day: 0, lost_exp_per_day: 0 },
        ],
    },
    unrotated: { verdict: "depletes", horizon_hours: 168, depleted: UNROTATED_DEPLETED, dorm_overflow: 0, timeline: [] },
    morale_synced_hours_ago: 3.5,
    trainer_hints: [
        { operator: ROSMONTIS, value_pct: 30 },
        { operator: ACIDDROP, value_pct: 30 },
        { operator: FIREWATCH, value_pct: 30 },
    ],
    drones: { current: 173.5, max: 250, full_in_hours: 7.65 },
};

/** The same base after the optimizer's proposal is applied: Bison replaces Jaye, the weaker factories restaffed. */
const OPTIMIZED: IEvaluateResponse = {
    ...EVALUATION,
    assignment: {
        rooms: [
            TRADING_ROOMS[0],
            { ...TRADING_ROOMS[1], total_efficiency: 100, operators: [FIAMMETTA, SHAMARE, BISON], yield_lmd_per_day: 20_000, capacity: 14, fill_hours: 12.6 },
            FACTORY_ROOMS[0],
            { ...FACTORY_ROOMS[1], total_efficiency: 118, yield_gold_per_day: 43.6, fill_hours: 29.7 },
            FACTORY_ROOMS[2],
            { ...FACTORY_ROOMS[3], total_efficiency: 96, yield_exp_per_day: 15_680, fill_hours: 82.7 },
            ...OTHER_ROOMS,
        ],
        total_production_efficiency: 636,
        yield_lmd_per_day: 40_000,
        yield_exp_per_day: 31_040,
        yield_total_value: 71_040,
    },
};

/** 15 samples = t0 plus every 12h boundary of a 168h week; the last one is `end`. */
const tl = (operator: { operator_id: string; name: string }, room_type: string, slot_id: string, samples: number[]): IMoraleTimeline => ({ operator, room_type, slot_id, samples, end: samples[samples.length - 1] });
const TWO_SHIFT = [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 12];

/** The morale-over-time chart's rows, most-at-risk first. */
const TIMELINE: IMoraleTimeline[] = [
    tl(VERMEIL, "MANUFACTURE", "slot_7", [24, 9, 0.5, 12.5, 24, 9, 0.5, 12.5, 24, 9, 0.5, 12.5, 24, 9, 3.1]),
    tl(EXUSIAI, "TRADING", "slot_5", [24, 11, 2, 14, 24, 11, 2, 14, 24, 11, 2, 14, 24, 11, 5.8]),
    tl(PERFUMER, "MANUFACTURE", "slot_15", [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 9.4]),
    tl(TEXAS, "TRADING", "slot_5", TWO_SHIFT),
    tl(SHAMARE, "TRADING", "slot_6", [24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 14]),
    tl(SCAVENGER, "MANUFACTURE", "slot_7", [24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 15.8]),
    tl(AMIYA, "CONTROL", "slot_34", [24, 23.4, 22.8, 22.2, 21.6, 21, 20.4, 19.8, 19.2, 18.6, 18, 17.4, 16.8, 16.2, 15.6]),
    tl(GREYY, "POWER", "slot_24", [24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 18.2]),
];

/** Seven simulated days of the recommended rotation, in the shape the planner returns. */
const ROTATION: IRotationResponse = {
    shift_count: 2,
    rotation: {
        shifts: [],
        sustained: [FIAMMETTA],
        sustainability: {
            verdict: "depletes",
            horizon_hours: 168,
            depleted: [
                { operator: VERMEIL, at_hours: 31.2, room_type: "MANUFACTURE" },
                { operator: WEEDY, at_hours: 100.5, room_type: "MANUFACTURE" },
            ],
            dorm_overflow: 0,
            timeline: TIMELINE,
            facilities: [
                { slot_id: "slot_5", room_type: "TRADING", formula_type: null, lmd: 140_000, gold: 0, exp: 0, idle_hours: 0 },
                { slot_id: "slot_6", room_type: "TRADING", formula_type: null, lmd: 128_075, gold: 0, exp: 0, idle_hours: 4.5 },
                { slot_id: "slot_7", room_type: "MANUFACTURE", formula_type: "F_GOLD", lmd: 0, gold: 273, exp: 0, idle_hours: 0 },
                { slot_id: "slot_14", room_type: "MANUFACTURE", formula_type: "F_GOLD", lmd: 0, gold: 270, exp: 0, idle_hours: 6 },
                { slot_id: "slot_15", room_type: "MANUFACTURE", formula_type: "F_EXP", lmd: 0, gold: 0, exp: 107_520, idle_hours: 0 },
                { slot_id: "slot_16", room_type: "MANUFACTURE", formula_type: "F_EXP", lmd: 0, gold: 0, exp: 94_043, idle_hours: 9.5 },
            ],
        },
    },
};

/** The optimizer's answer: Bison for Jaye in the second post and the two weaker factories restaffed - scored against the draft by the same engine run. */
const PROPOSAL: IOptimizeResponse = {
    baseline: EVALUATION.assignment,
    proposal: {
        rooms: OPTIMIZED.assignment.rooms,
        total_production_efficiency: 636,
        yield_lmd_per_day: 40_000,
        yield_exp_per_day: 31_040,
        yield_total_value: 71_040,
    },
    room_diffs: [
        { slot_id: "slot_6", room_type: "TRADING", before: [FIAMMETTA.operator_id, SHAMARE.operator_id, JAYE.operator_id], after: [FIAMMETTA.operator_id, SHAMARE.operator_id, BISON.operator_id], efficiency_before: 88, efficiency_after: 100, yield_before: 18_800, yield_after: 20_000 },
        { slot_id: "slot_14", room_type: "MANUFACTURE", before: [GUMMY.operator_id, CEOBE.operator_id, CUORA.operator_id], after: [GUMMY.operator_id, CEOBE.operator_id, "char_4105_almond"], efficiency_before: 100, efficiency_after: 118, yield_before: 20_000, yield_after: 21_800 },
        { slot_id: "slot_16", room_type: "MANUFACTURE", before: [WEEDY.operator_id, DOROTHY.operator_id, SILENCE.operator_id], after: [WEEDY.operator_id, DOROTHY.operator_id, "char_236_rope"], efficiency_before: 78, efficiency_after: 96, yield_before: 14_240, yield_after: 15_680 },
        // A room whose crew is unchanged is filtered out of "Rooms restaffed".
        { slot_id: "slot_5", room_type: "TRADING", before: [TEXAS.operator_id, LAPPLAND.operator_id, EXUSIAI.operator_id], after: [TEXAS.operator_id, LAPPLAND.operator_id, EXUSIAI.operator_id], efficiency_before: 100, efficiency_after: 100, yield_before: 20_000, yield_after: 20_000 },
    ],
    power: { generated: 810, consumed: 790, net: 20 },
};

/** An `IOptimizerAPI` for a scored draft; only the fields StatsForNerds reads carry data. */
function optimizerApi(overrides: Partial<IOptimizerAPI>): IOptimizerAPI {
    return {
        layout: [],
        dirty: false,
        catalog: CATALOG,
        slots: [],
        formulas: [],
        presets: [],
        shiftCount: 2,
        boardRooms: [],
        catalogLoading: false,
        layoutLoading: false,
        evaluation: EVALUATION,
        evaluating: false,
        evaluationError: null,
        rotation: ROTATION,
        rotationLoading: false,
        ignorePromotion: false,
        setIgnorePromotion: noop,
        openRecruitSlots: 0,
        setOpenRecruitSlots: noop,
        factsSaved: null,
        trainingClass: "Sniper",
        setTrainingClass: noop,
        claimIntervalHours: undefined,
        setClaimIntervalHours: noop,
        viewShift: null,
        setViewShift: noop,
        shiftRoom: () => undefined,
        proposal: PROPOSAL,
        optimizing: false,
        optimizeError: null,
        runOptimize: noop,
        reset: noop,
        ...overrides,
    };
}

/** The panel as BasePanel mounts it; `open` clicks the trigger on mount, since the open state is internal and closed by default. */
function Stage({ api, open = false }: { api: IOptimizerAPI; open?: boolean }) {
    const stage = useRef<HTMLDivElement>(null);
    useEffect(() => {
        if (!open) return;
        // The click has to wait two frames after mount; firing it from the effect is dropped.
        let frame = requestAnimationFrame(() => {
            frame = requestAnimationFrame(() => {
                stage.current?.querySelector<HTMLButtonElement>('button[aria-expanded="false"]')?.click();
                frame = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur?.());
            });
        });
        return () => cancelAnimationFrame(frame);
    }, [open]);
    return (
        <div className="max-w-4xl" ref={stage}>
            <BaseOptimizerProvider value={api}>
                <StatsForNerds />
            </BaseOptimizerProvider>
        </div>
    );
}

/** Opened after an optimizer run and a rotation simulation: all nine sections, the morale chart included. */
export const AllSections = () => <Stage api={optimizerApi({})} open />;

/** Opened on a freshly scored draft - no proposal and no rotation yet, so only the evaluation's four sections and the dorm table show. */
export const EvaluationOnly = () => <Stage api={optimizerApi({ proposal: null, rotation: null })} open />;

/** At rest: the collapsed trigger BasePanel shows under the Deep Dive. */
export const Collapsed = () => <Stage api={optimizerApi({})} />;
