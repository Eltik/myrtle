import { BasePanel } from "frontend";
import type { ReactNode } from "react";

const BASE_ACCENT = "oklch(0.70 0.16 145)";

/** The expanded Base subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Base · Drone &amp; facility upgrades</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: BASE_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

// ⚠️ `operators` is deliberately left empty on every room below. The inline
// panel this preview grades never draws an operator chip — the room-by-room
// detail lives in the full-plan dialog and in the off-screen export poster
// `BasePanel` always keeps mounted at `-left-24999.75`. Avatars in that poster
// are `loading="lazy"` and can never intersect the viewport, so `img.decode()`
// in the capture harness's `settle()` never resolves and the capture hangs
// forever. Empty crews keep the visible card identical and the capture finite.
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


const layout = [
    { room_type: "CONTROL", count: 1, levels: [5] },
    { room_type: "TRADING", count: 2, levels: [3, 3] },
    { room_type: "MANUFACTURE", count: 4, levels: [3, 3, 3, 2] },
    { room_type: "POWER", count: 3, levels: [3, 3, 2] },
    { room_type: "DORMITORY", count: 4, levels: [5, 5, 4, 3] },
    { room_type: "WORKSHOP", count: 1, levels: [3] },
    { room_type: "MEETING", count: 1, levels: [3] },
];

const current = {
    rooms: [
        room("slot_1_1", "CONTROL", null, 15, [], {}, { non_production: [{ room_type: "MEETING", value: 15 }, { room_type: "HIRE", value: 10 }] }),
        room("slot_2_1", "TRADING", null, 70, [], { lmd: 25700 }, { locked: true }),
        room("slot_2_2", "TRADING", null, 42, [], { lmd: 15200 }),
        room("slot_3_1", "MANUFACTURE", "F_GOLD", 55, [], { gold: 47 }),
        room("slot_3_3", "MANUFACTURE", "F_EXP", 48, [], { exp: 8400 }),
    ],
    total_production_efficiency: 230,
    yield_lmd_per_day: 40900,
    yield_exp_per_day: 8400,
    yield_total_value: 49300,
};

const optimal = {
    rooms: [
        room("slot_1_1", "CONTROL", null, 27, [], {}, { non_production: [{ room_type: "MEETING", value: 20 }, { room_type: "HIRE", value: 15 }] }),
        room("slot_2_1", "TRADING", null, 85, [], { lmd: 31200 }, { locked: true }),
        room("slot_2_2", "TRADING", null, 65, [], { lmd: 22400 }),
        room("slot_3_1", "MANUFACTURE", "F_GOLD", 75, [], { gold: 64 }),
        room("slot_3_3", "MANUFACTURE", "F_EXP", 72, [], { exp: 12600 }),
    ],
    total_production_efficiency: 324,
    yield_lmd_per_day: 53600,
    yield_exp_per_day: 12600,
    yield_total_value: 66200,
};

const syncedBase = { base: { layout, current, optimal, rotation: null, shift_rotation: null, perception: null } };

const noBase = { base: { layout: [], current: null, optimal: null, rotation: null, shift_rotation: null, perception: null } };

export const BasePlanSummary = () => (
    <PanelFrame pct="94.1%">
        <BasePanel improvements={syncedBase} accent={BASE_ACCENT} />
    </PanelFrame>
);

// Nothing synced from the RIIC yet, so there is no layout and no optimizer run
// to compare against — the panel degrades to a single hint.
export const NoBaseSynced = () => (
    <PanelFrame pct="0.0%">
        <BasePanel improvements={noBase} accent={BASE_ACCENT} />
    </PanelFrame>
);
