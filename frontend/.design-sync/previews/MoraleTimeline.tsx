import { MoraleTimeline } from "frontend";
import type { ReactNode } from "react";
import type { IMoraleTimeline } from "../../src/lib/api/user";
import type { Catalog } from "../../src/lib/base/catalog";

// MoraleTimeline is the "morale over time" chart at the bottom of Stats for
// Nerds: one card per simulated operator, most-at-risk first, with a 96x26
// sparkline of `samples` (morale at every 12h boundary of the 7-day rotation
// simulation, samples[0] = 24) and the bar they END the week on. An end below
// 12/24 paints the figure red. `catalog` only supplies room display names;
// `room_type` colours the room label through `roomAccent`.

const phases = (stationed: number, power: number[]) => power.map((electricity, i) => ({ level: i + 1, max_stationed: stationed, electricity, manpower_cost: 100 }));

/** The catalog rooms the timeline can name (`building_data.json`, EN). */
const CATALOG: Catalog = new Map([
    ["CONTROL", { room_type: "CONTROL", name: "Control Center", category: "SPECIAL", max_count: 1, size_col: 4, size_row: 1, phases: phases(5, [0, 0, 0, 0, 0]) }],
    ["TRADING", { room_type: "TRADING", name: "Trading Post", category: "OUTPUT", max_count: 5, size_col: 3, size_row: 1, phases: phases(3, [-10, -30, -60]) }],
    ["MANUFACTURE", { room_type: "MANUFACTURE", name: "Factory", category: "OUTPUT", max_count: 5, size_col: 3, size_row: 1, phases: phases(3, [-10, -30, -60]) }],
    ["POWER", { room_type: "POWER", name: "Power Plant", category: "OUTPUT", max_count: 3, size_col: 3, size_row: 1, phases: phases(1, [60, 130, 270]) }],
    ["DORMITORY", { room_type: "DORMITORY", name: "Dormitory", category: "CUSTOM", max_count: 4, size_col: 5, size_row: 1, phases: phases(5, [-10, -20, -30, -45, -65]) }],
    ["MEETING", { room_type: "MEETING", name: "Reception Room", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(2, [-10, -30, -60]) }],
    ["HIRE", { room_type: "HIRE", name: "Office", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(1, [-10, -30, -60]) }],
    ["TRAINING", { room_type: "TRAINING", name: "Training Room", category: "FUNCTION", max_count: 1, size_col: 3, size_row: 1, phases: phases(2, [-10, -30, -60]) }],
]);

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** 15 samples = t0 plus every 12h boundary of a 168h week; the last one is `end`. */
const row = (operator: { operator_id: string; name: string }, room_type: string, slot_id: string, samples: number[]): IMoraleTimeline => ({ operator, room_type, slot_id, samples, end: samples[samples.length - 1] });

/** Work 12h (drain 1/h), rest 12h back to full: the steady two-shift rhythm. */
const TWO_SHIFT = [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 12];
/** A 24h production block on a 1.25/h drain, then a 12h rest that never quite refills. */
const HEAVY_BLOCK = [24, 9, 0.5, 12.5, 24, 9, 0.5, 12.5, 24, 9, 0.5, 12.5, 24, 9, 3.1];
/** Control Center: 0.05/h, a slow slide across the whole week. */
const CC_SLIDE = [24, 23.4, 22.8, 22.2, 21.6, 21, 20.4, 19.8, 19.2, 18.6, 18, 17.4, 16.8, 16.2, 15.6];

/** The week the planner simulates for a 2-4-3 base: most-at-risk first, the two heavy blocks end in the red. */
export const MostAtRiskFirst = () => (
    <Stage title="Morale over time">
        <MoraleTimeline
            catalog={CATALOG}
            timeline={[
                row(op("char_190_clour", "Vermeil"), "MANUFACTURE", "slot_7", HEAVY_BLOCK),
                row(op("char_103_angel", "Exusiai"), "TRADING", "slot_5", [24, 11, 2, 14, 24, 11, 2, 14, 24, 11, 2, 14, 24, 11, 5.8]),
                row(op("char_181_flower", "Perfumer"), "MANUFACTURE", "slot_15", [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 9.4]),
                row(op("char_102_texas", "Texas"), "TRADING", "slot_5", TWO_SHIFT),
                row(op("char_254_vodfox", "Shamare"), "TRADING", "slot_6", [24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 24, 14, 14]),
                row(op("char_149_scave", "Scavenger"), "MANUFACTURE", "slot_7", [24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 24, 12.5, 15.8]),
                row(op("char_002_amiya", "Amiya"), "CONTROL", "slot_34", CC_SLIDE),
                row(op("char_253_greyy", "Greyy"), "POWER", "slot_24", [24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 18.2]),
            ]}
        />
    </Stage>
);

/** A rotation that holds up: every bar ends at or above half, including a permanent dorm resident who never leaves 24. */
export const EveryoneHoldsUp = () => (
    <Stage title="Morale over time">
        <MoraleTimeline
            catalog={CATALOG}
            timeline={[
                row(op("char_102_texas", "Texas"), "TRADING", "slot_5", TWO_SHIFT),
                row(op("char_140_whitew", "Lappland"), "TRADING", "slot_5", [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 16]),
                row(op("char_128_plosis", "Ptilopsis"), "MANUFACTURE", "slot_15", [24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 20]),
                row(op("char_277_sqrrel", "Shaw"), "POWER", "slot_25", [24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 24, 13, 22.5]),
                row(op("char_2023_ling", "Ling"), "DORMITORY", "slot_3", [24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24, 24]),
                row(op("char_002_amiya", "Amiya"), "CONTROL", "slot_34", CC_SLIDE),
            ]}
        />
    </Stage>
);

/** Two operators hit zero mid-week and sit dry until their rest: the flat floors are the lost hours the deep dive reports. */
export const RunsDry = () => (
    <Stage title="Morale over time">
        <MoraleTimeline
            catalog={CATALOG}
            timeline={[
                row(op("char_400_weedy", "Weedy"), "MANUFACTURE", "slot_16", [24, 9, 0, 0, 12, 24, 9, 0, 0, 12, 24, 9, 0, 0, 0]),
                row(op("char_272_strong", "Jaye"), "TRADING", "slot_6", [24, 10, 0, 0, 14, 24, 10, 0, 0, 14, 24, 10, 0, 0, 2.4]),
                row(op("char_163_hpsts", "Vulcan"), "MANUFACTURE", "slot_15", [24, 12, 0, 12, 24, 12, 0, 12, 24, 12, 0, 12, 24, 12, 6]),
                row(op("char_4048_doroth", "Dorothy"), "MANUFACTURE", "slot_16", [24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 24, 12, 12]),
            ]}
        />
    </Stage>
);

/** The Stats-for-Nerds section the chart lives in: a card with the uppercase kicker above the grid. */
function Stage({ title, children }: { title: string; children: ReactNode }) {
    return (
        <div className="max-w-3xl rounded-xl border border-border bg-card px-4 py-3">
            <section className="flex flex-col">
                <h3 className="mb-1 font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{title}</h3>
                <div className="divide-y divide-border/60">{children}</div>
            </section>
        </div>
    );
}
