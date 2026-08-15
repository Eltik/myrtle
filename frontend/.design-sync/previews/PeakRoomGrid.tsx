import type { CSSProperties } from "react";
import { PeakRoomGrid } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

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

const room = (r: RoomInit) => ({
    level: 3,
    formula_type: null,
    total_efficiency: 0,
    order_value: 0,
    locked: false,
    operators: [],
    yield_lmd_per_day: 0,
    yield_gold_per_day: 0,
    yield_exp_per_day: 0,
    non_production: [],
    ...r,
});

const assignment = (rooms: ReturnType<typeof room>[], lmd: number, exp: number, eff: number) => ({
    rooms,
    total_production_efficiency: eff,
    yield_lmd_per_day: lmd,
    yield_exp_per_day: exp,
    yield_total_value: lmd + exp,
});

const FULL_BASE = assignment(
    [
        room({ slot_id: "slot_22", room_type: "TRADING", total_efficiency: 105, order_value: 30, operators: [op("char_102_texas", "Texas"), op("char_103_angel", "Exusiai"), op("char_140_whitew", "Lappland")], yield_lmd_per_day: 25_700 }),
        room({ slot_id: "slot_23", room_type: "TRADING", total_efficiency: 78, operators: [op("char_476_blkngt", "Blacknight"), op("char_1028_texas2", "Texas the Omertosa")], yield_lmd_per_day: 15_500 }),
        room({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", total_efficiency: 130, locked: true, operators: [op("char_190_clour", "Vermeil"), op("char_149_scave", "Scavenger"), op("char_210_stward", "Steward")], yield_gold_per_day: 1_560 }),
        room({ slot_id: "slot_13", room_type: "MANUFACTURE", formula_type: "F_EXP", total_efficiency: 95, operators: [op("char_164_nightm", "Nightmare"), op("char_290_vigna", "Vigna")], yield_exp_per_day: 4_200 }),
        room({ slot_id: "slot_31", room_type: "POWER", total_efficiency: 15, operators: [op("char_253_greyy", "Greyy")] }),
        room({ slot_id: "slot_02", room_type: "CONTROL", level: 5, total_efficiency: 35, operators: [op("char_003_kalts", "Kal'tsit"), op("char_128_plosis", "Ptilopsis"), op("char_002_amiya", "Amiya")], non_production: [{ room_type: "MEETING", value: 15 }, { room_type: "HIRE", value: 10 }] }),
    ],
    41_200,
    8_640,
    118.5,
);

/** The dialog layout: the best single-snapshot staffing per room, two columns wide. */
export const OptimalArrangement = () => (
    <div className="max-w-2xl" style={ACCENT}>
        <PeakRoomGrid optimal={FULL_BASE} interactive className="grid-cols-1 sm:grid-cols-2" />
    </div>
);

/** The narrow single-column stack the dialog falls back to on small screens. */
export const SingleColumn = () => (
    <div className="max-w-xs" style={ACCENT}>
        <PeakRoomGrid
            optimal={assignment(
                [
                    room({ slot_id: "slot_22", room_type: "TRADING", total_efficiency: 105, order_value: 30, operators: [op("char_102_texas", "Texas"), op("char_103_angel", "Exusiai")], yield_lmd_per_day: 25_700 }),
                    room({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", total_efficiency: 130, operators: [op("char_190_clour", "Vermeil"), op("char_210_stward", "Steward")], yield_gold_per_day: 1_560 }),
                    room({ slot_id: "slot_31", room_type: "POWER", total_efficiency: 15, operators: [op("char_253_greyy", "Greyy")] }),
                ],
                25_700,
                0,
                86.4,
            )}
            className="grid-cols-1"
        />
    </div>
);

/** Rooms the plan leaves alone: an unstaffed slot, and an Office reserved purely to
 *  feed the resource pool — tagged "economy" instead of a misleading +0%. */
export const PartiallyStaffed = () => (
    <div className="max-w-2xl" style={ACCENT}>
        <PeakRoomGrid
            optimal={assignment(
                [
                    room({ slot_id: "slot_22", room_type: "TRADING", total_efficiency: 62, operators: [op("char_102_texas", "Texas"), op("char_124_kroos", "Kroos")], yield_lmd_per_day: 12_400 }),
                    room({ slot_id: "slot_23", room_type: "TRADING", total_efficiency: 0 }),
                    room({ slot_id: "slot_41", room_type: "HIRE", total_efficiency: 0, operators: [op("char_254_vodfox", "Shamare")] }),
                    room({ slot_id: "slot_42", room_type: "MEETING", total_efficiency: 0, operators: [op("char_4045_heidi", "Heidi")] }),
                ],
                12_400,
                0,
                62,
            )}
            className="grid-cols-1 sm:grid-cols-2"
        />
    </div>
);
