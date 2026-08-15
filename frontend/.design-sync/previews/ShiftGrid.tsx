import type { CSSProperties } from "react";
import { ShiftGrid } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

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

const SHIFTS = [
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
];

/** The dialog layout: three shifts side by side, one column each. */
export const ThreeShifts = () => (
    <div className="max-w-3xl" style={ACCENT}>
        <ShiftGrid rotation={{ shifts: SHIFTS, sustained: [OMERTOSA], sustainability: null }} interactive className="grid-cols-1 sm:grid-cols-3" />
    </div>
);

/** The export poster passes a fixed three-column grid and no interactivity, so the
 *  captured PNG matches the dialog exactly. */
export const ExportColumns = () => (
    <div className="max-w-3xl" style={ACCENT}>
        <ShiftGrid rotation={{ shifts: SHIFTS, sustained: [], sustainability: null }} className="grid-cols-3" />
    </div>
);

/** The narrow stack the dialog falls back to below the `sm` breakpoint. */
export const StackedOnMobile = () => (
    <div className="w-72" style={ACCENT}>
        <ShiftGrid rotation={{ shifts: SHIFTS.slice(0, 2), sustained: [OMERTOSA], sustainability: null }} interactive className="grid-cols-1" />
    </div>
);
