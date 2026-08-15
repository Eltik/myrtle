import type { CSSProperties } from "react";
import { PeakGrid } from "frontend";

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

/** The "Peak single-shift" section of the base plan: the yield bar plus the room grid. */
export const PeakSingleShift = () => (
    <div className="max-w-2xl" style={ACCENT}>
        <PeakGrid
            optimal={assignment(
                [
                    room({ slot_id: "slot_22", room_type: "TRADING", total_efficiency: 105, order_value: 30, operators: [op("char_102_texas", "Texas"), op("char_103_angel", "Exusiai"), op("char_140_whitew", "Lappland")], yield_lmd_per_day: 25_700 }),
                    room({ slot_id: "slot_23", room_type: "TRADING", total_efficiency: 78, operators: [op("char_476_blkngt", "Blacknight"), op("char_1028_texas2", "Texas the Omertosa")], yield_lmd_per_day: 15_500 }),
                    room({ slot_id: "slot_12", room_type: "MANUFACTURE", formula_type: "F_GOLD", total_efficiency: 130, locked: true, operators: [op("char_190_clour", "Vermeil"), op("char_149_scave", "Scavenger"), op("char_210_stward", "Steward")], yield_gold_per_day: 1_560 }),
                    room({ slot_id: "slot_13", room_type: "MANUFACTURE", formula_type: "F_EXP", total_efficiency: 95, operators: [op("char_164_nightm", "Nightmare"), op("char_290_vigna", "Vigna")], yield_exp_per_day: 4_200 }),
                ],
                41_200,
                8_640,
                118.5,
            )}
        />
    </div>
);

/** The Control Center and its support rooms: a global buff, non-production effects in
 *  each boosted facility's own units, and the "economy" tag on a reserved Office. */
export const SupportRooms = () => (
    <div className="max-w-2xl" style={ACCENT}>
        <PeakGrid
            optimal={assignment(
                [
                    room({ slot_id: "slot_02", room_type: "CONTROL", level: 5, total_efficiency: 35, operators: [op("char_003_kalts", "Kal'tsit"), op("char_128_plosis", "Ptilopsis"), op("char_002_amiya", "Amiya"), op("char_130_doberm", "Dobermann")], non_production: [{ room_type: "MEETING", value: 15 }, { room_type: "TRAINING", value: 12 }, { room_type: "HIRE", value: 10 }] }),
                    room({ slot_id: "slot_31", room_type: "POWER", total_efficiency: 15, operators: [op("char_253_greyy", "Greyy")] }),
                    room({ slot_id: "slot_41", room_type: "HIRE", total_efficiency: 0, operators: [op("char_254_vodfox", "Shamare")] }),
                    room({ slot_id: "slot_42", room_type: "MEETING", total_efficiency: 0, operators: [op("char_4045_heidi", "Heidi"), op("char_298_susuro", "Sussurro")] }),
                ],
                6_800,
                0,
                35,
            )}
        />
    </div>
);

/** A freshly synced base with only two rooms staffed — one still empty. */
export const EarlyBase = () => (
    <div className="max-w-2xl" style={ACCENT}>
        <PeakGrid
            optimal={assignment(
                [
                    room({ slot_id: "slot_22", room_type: "TRADING", level: 2, total_efficiency: 40, operators: [op("char_102_texas", "Texas")], yield_lmd_per_day: 7_200 }),
                    room({ slot_id: "slot_12", room_type: "MANUFACTURE", level: 2, formula_type: "F_GOLD", total_efficiency: 55, operators: [op("char_210_stward", "Steward"), op("char_190_clour", "Vermeil")], yield_gold_per_day: 720 }),
                    room({ slot_id: "slot_13", room_type: "MANUFACTURE", level: 1, formula_type: "F_EXP", total_efficiency: 0 }),
                ],
                7_200,
                0,
                47.5,
            )}
        />
    </div>
);
