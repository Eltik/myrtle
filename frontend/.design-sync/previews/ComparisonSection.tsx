import { ComparisonSection } from "frontend";
import type { CSSProperties, ReactNode } from "react";

const BASE_ACCENT = "oklch(0.70 0.16 145)";

/**
 * `BasePanel` publishes the panel accent as `--imp-accent` on the subtree, and
 * every figure in this section colours itself from that variable — so the stage
 * has to set it exactly like the real parent does.
 */
const PlanFrame = ({ title, children }: { title: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl rounded-xl border border-border bg-card p-4 sm:p-5" style={{ "--imp-accent": BASE_ACCENT } as CSSProperties}>
        <div className="mb-3 flex items-center justify-between border-border/40 border-b pb-1.5">
            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em]" style={{ color: `color-mix(in oklch, ${BASE_ACCENT} 60%, var(--foreground))` }}>
                {title}
            </span>
            <span className="rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">Base plan</span>
        </div>
        {children}
    </div>
);

const op = (operator_id: string, name: string) => ({ operator_id, name });

const room = (slot_id: string, room_type: string, formula_type: string | null, total_efficiency: number, operators: { operator_id: string; name: string }[], yields: { lmd?: number; gold?: number; exp?: number }, extra?: { locked?: boolean; non_production?: { room_type: string; value: number }[] }) => ({
    slot_id,
    room_type,
    level: 3,
    formula_type,
    total_efficiency,
    order_value: 0,
    locked: extra?.locked ?? false,
    operators,
    yield_lmd_per_day: yields.lmd ?? 0,
    yield_gold_per_day: yields.gold ?? 0,
    yield_exp_per_day: yields.exp ?? 0,
    non_production: extra?.non_production ?? [],
});

const AMIYA = op("char_002_amiya", "Amiya");
const SWIRE = op("char_308_swire", "Swire");
const ANGELINA = op("char_291_aglina", "Angelina");
const KALTSIT = op("char_003_kalts", "Kal'tsit");
const TEXAS = op("char_102_texas", "Texas");
const EXUSIAI = op("char_103_angel", "Exusiai");
const LAPPLAND = op("char_140_whitew", "Lappland");
const JAYE = op("char_272_strong", "Jaye");
const PROVISO = op("char_4032_provs", "Proviso");
const BIBEAK = op("char_252_bibeak", "Bibeak");
const VIGNA = op("char_290_vigna", "Vigna");
const BEANSTALK = op("char_452_bstalk", "Beanstalk");
const CROISSANT = op("char_201_moeshd", "Croissant");
const SCENE = op("char_336_folivo", "Scene");
const MULBERRY = op("char_473_mberry", "Mulberry");
const CLIFFHEART = op("char_173_slchan", "Cliffheart");

const current = {
    rooms: [
        room("slot_1_1", "CONTROL", null, 15, [AMIYA, SWIRE], {}, { non_production: [{ room_type: "MEETING", value: 15 }, { room_type: "HIRE", value: 10 }] }),
        room("slot_2_1", "TRADING", null, 70, [TEXAS, EXUSIAI, LAPPLAND], { lmd: 25700 }, { locked: true }),
        room("slot_2_2", "TRADING", null, 42, [JAYE], { lmd: 15200 }),
        room("slot_3_1", "MANUFACTURE", "F_GOLD", 55, [VIGNA, BEANSTALK], { gold: 47 }),
        room("slot_3_2", "MANUFACTURE", "F_GOLD", 30, [CROISSANT], { gold: 26 }),
        room("slot_3_3", "MANUFACTURE", "F_EXP", 48, [SCENE], { exp: 8400 }),
        room("slot_3_4", "MANUFACTURE", null, 0, [], {}),
    ],
    total_production_efficiency: 260,
    yield_lmd_per_day: 40900,
    yield_exp_per_day: 8400,
    yield_total_value: 49300,
};

const optimal = {
    rooms: [
        room("slot_1_1", "CONTROL", null, 27, [AMIYA, SWIRE, KALTSIT], {}, { non_production: [{ room_type: "MEETING", value: 20 }, { room_type: "HIRE", value: 15 }, { room_type: "TRAINING", value: 10 }] }),
        room("slot_2_1", "TRADING", null, 85, [TEXAS, EXUSIAI, LAPPLAND], { lmd: 31200 }, { locked: true }),
        room("slot_2_2", "TRADING", null, 65, [JAYE, PROVISO, BIBEAK], { lmd: 22400 }),
        room("slot_3_1", "MANUFACTURE", "F_GOLD", 75, [VIGNA, BEANSTALK, ANGELINA], { gold: 64 }),
        room("slot_3_2", "MANUFACTURE", "F_GOLD", 60, [CROISSANT, CLIFFHEART], { gold: 51 }),
        room("slot_3_3", "MANUFACTURE", "F_EXP", 72, [SCENE, MULBERRY], { exp: 12600 }),
        room("slot_3_4", "MANUFACTURE", "F_EXP", 45, [BEANSTALK], { exp: 7900 }),
    ],
    total_production_efficiency: 429,
    yield_lmd_per_day: 53600,
    yield_exp_per_day: 20500,
    yield_total_value: 74100,
};

export const CurrentVsOptimal = () => (
    <PlanFrame title="Current vs optimal detail">
        <ComparisonSection current={current} optimal={optimal} />
    </PlanFrame>
);

// Already running the optimizer's peak staffing: every delta reads zero and the
// "move here" outlines disappear, which is the state a finished plan lands in.
export const AlreadyOptimal = () => (
    <PlanFrame title="Current vs optimal detail">
        <ComparisonSection current={optimal} optimal={optimal} />
    </PlanFrame>
);
