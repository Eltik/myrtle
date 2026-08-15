import type { CSSProperties, ReactNode } from "react";
import { ExportPlanContent } from "frontend";

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The poster lays out at a fixed 960px so the exported PNG is a wide sheet rather
 *  than a tall ribbon; `zoom` scales the whole thing down to fit this card. */
const Poster = ({ children }: { children: ReactNode }) => (
    <div className="w-fit overflow-hidden rounded-md border border-border/40" style={{ zoom: 0.85, "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties}>
        {children}
    </div>
);

const TEXAS = op("char_102_texas", "Texas");
const EXUSIAI = op("char_103_angel", "Exusiai");
const LAPPLAND = op("char_140_whitew", "Lappland");
const KROOS = op("char_124_kroos", "Kroos");
const BLACKNIGHT = op("char_476_blkngt", "Blacknight");
const OMERTOSA = op("char_1028_texas2", "Texas the Omertosa");
const VERMEIL = op("char_190_clour", "Vermeil");
const SCAVENGER = op("char_149_scave", "Scavenger");
const STEWARD = op("char_210_stward", "Steward");
const NIGHTMARE = op("char_164_nightm", "Nightmare");
const VIGNA = op("char_290_vigna", "Vigna");
const SPECTER = op("char_143_ghost", "Specter");
const KALTSIT = op("char_003_kalts", "Kal'tsit");
const PTILOPSIS = op("char_128_plosis", "Ptilopsis");
const AMIYA = op("char_002_amiya", "Amiya");
const DOBERMANN = op("char_130_doberm", "Dobermann");
const GREYY = op("char_253_greyy", "Greyy");

const LAYOUT = [
    { room_type: "TRADING", count: 2, levels: [3, 3] },
    { room_type: "MANUFACTURE", count: 4, levels: [3, 3, 2, 2] },
    { room_type: "POWER", count: 3, levels: [3, 3, 2] },
    { room_type: "DORMITORY", count: 4, levels: [5, 4, 3, 1] },
    { room_type: "CONTROL", count: 1, levels: [5] },
    { room_type: "MEETING", count: 1, levels: [3] },
];

interface ShiftRoomInit {
    slot_id: string;
    room_type: string;
    formula_type?: string | null;
    active?: boolean;
    recommended?: { operator_id: string; name: string }[];
    current?: { operator_id: string; name: string }[];
    swap_in?: { operator_id: string; name: string }[];
    swap_out?: { operator_id: string; name: string }[];
    matches?: boolean;
    equivalent?: boolean;
    gap_pct?: number | null;
    efficiency?: number | null;
    team_id?: string | null;
    team_label?: string | null;
}

const shiftRoom = (r: ShiftRoomInit) => ({ formula_type: null, active: true, recommended: [], current: [], swap_in: [], swap_out: [], moved_out: [], matches: false, equivalent: false, gap_pct: null, efficiency: null, team_id: null, team_label: null, ...r });

const SHIFT_ROTATION = {
    shifts: [
        {
            index: 1,
            rooms: [
                shiftRoom({ slot_id: "slot_22", room_type: "TRADING", recommended: [TEXAS, EXUSIAI, LAPPLAND], current: [TEXAS, KROOS], swap_in: [EXUSIAI, LAPPLAND], swap_out: [KROOS], efficiency: 105, team_id: "trade-a", team_label: "Team A" }),
                shiftRoom({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", recommended: [VERMEIL, SCAVENGER, STEWARD], current: [VERMEIL, SCAVENGER, STEWARD], matches: true, efficiency: 130, team_id: "fact-a", team_label: "Team A" }),
                shiftRoom({ slot_id: "slot_02", room_type: "CONTROL", recommended: [KALTSIT, PTILOPSIS, AMIYA], current: [KALTSIT], swap_in: [PTILOPSIS, AMIYA], efficiency: 35, team_id: "cc-1", team_label: "Squad 1" }),
            ],
        },
        {
            index: 2,
            rooms: [
                shiftRoom({ slot_id: "slot_22", room_type: "TRADING", recommended: [TEXAS, EXUSIAI, LAPPLAND], current: [TEXAS, EXUSIAI, KROOS], equivalent: true, gap_pct: -3.1, efficiency: 102, team_id: "trade-a", team_label: "Team A" }),
                shiftRoom({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", recommended: [NIGHTMARE, VIGNA, SPECTER], swap_in: [NIGHTMARE, VIGNA, SPECTER], efficiency: 118, team_id: "fact-b", team_label: "Team B" }),
                shiftRoom({ slot_id: "slot_02", room_type: "CONTROL", active: false, current: [KALTSIT], team_id: "cc-1", team_label: "Squad 1" }),
            ],
        },
        {
            index: 3,
            rooms: [
                shiftRoom({ slot_id: "slot_22", room_type: "TRADING", recommended: [BLACKNIGHT, OMERTOSA], current: [KROOS], swap_in: [BLACKNIGHT, OMERTOSA], swap_out: [KROOS], efficiency: 78, team_id: "trade-b", team_label: "Team B" }),
                shiftRoom({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", recommended: [NIGHTMARE, VIGNA, SPECTER], current: [NIGHTMARE, VIGNA, SPECTER], matches: true, efficiency: 118, team_id: "fact-b", team_label: "Team B" }),
                shiftRoom({ slot_id: "slot_02", room_type: "CONTROL", recommended: [DOBERMANN, AMIYA], current: [DOBERMANN], swap_in: [AMIYA], efficiency: 28, team_id: "cc-2", team_label: "Squad 2" }),
            ],
        },
    ],
    sustained: [OMERTOSA],
    sustainability: { verdict: "holds_up", horizon_hours: 168, depleted: [], dorm_overflow: 0 },
};

interface RoomInit {
    slot_id: string;
    room_type: string;
    level?: number;
    formula_type?: string | null;
    total_efficiency?: number;
    order_value?: number;
    locked?: boolean;
    operators?: { operator_id: string; name: string }[];
    yield_lmd_per_day?: number;
    yield_gold_per_day?: number;
    yield_exp_per_day?: number;
    non_production?: { room_type: string; value: number }[];
}

const room = (r: RoomInit) => ({ level: 3, formula_type: null, total_efficiency: 0, order_value: 0, locked: false, operators: [], yield_lmd_per_day: 0, yield_gold_per_day: 0, yield_exp_per_day: 0, non_production: [], ...r });

const OPTIMAL = {
    rooms: [
        room({ slot_id: "slot_22", room_type: "TRADING", total_efficiency: 105, order_value: 30, operators: [TEXAS, EXUSIAI, LAPPLAND], yield_lmd_per_day: 25_700 }),
        room({ slot_id: "slot_23", room_type: "TRADING", total_efficiency: 78, operators: [BLACKNIGHT, OMERTOSA], yield_lmd_per_day: 15_500 }),
        room({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", total_efficiency: 130, locked: true, operators: [VERMEIL, SCAVENGER, STEWARD], yield_gold_per_day: 1_560 }),
        room({ slot_id: "slot_13", room_type: "MANUFACTURE", formula_type: "F_EXP", total_efficiency: 95, operators: [NIGHTMARE, VIGNA], yield_exp_per_day: 4_200 }),
        room({ slot_id: "slot_31", room_type: "POWER", total_efficiency: 15, operators: [GREYY] }),
        room({ slot_id: "slot_02", room_type: "CONTROL", level: 5, total_efficiency: 35, operators: [KALTSIT, PTILOPSIS, AMIYA], non_production: [{ room_type: "MEETING", value: 15 }, { room_type: "HIRE", value: 10 }] }),
    ],
    total_production_efficiency: 118.5,
    yield_lmd_per_day: 41_200,
    yield_exp_per_day: 8_640,
    yield_total_value: 49_840,
};

/** The downloadable poster: a title strip with the layout and daily yield, then the
 *  three shifts side by side. */
export const SharePoster = () => (
    <Poster>
        <ExportPlanContent base={{ current: null, optimal: OPTIMAL, rotation: null, layout: LAYOUT, shift_rotation: SHIFT_ROTATION, perception: null }} />
    </Poster>
);

/** No shift rotation available (a roster too thin to staff three shifts) — the poster
 *  falls back to the peak single-snapshot arrangement in two columns. */
export const PeakFallback = () => (
    <Poster>
        <ExportPlanContent base={{ current: null, optimal: OPTIMAL, rotation: null, layout: LAYOUT, shift_rotation: null, perception: null }} />
    </Poster>
);
