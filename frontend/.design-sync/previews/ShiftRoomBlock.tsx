import type { CSSProperties, ReactNode } from "react";
import { ShiftRoomBlock } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The shift column the cell always lives inside. */
const Column = ({ index, children }: { index: number; children: ReactNode }) => (
    <div className="w-80" style={ACCENT}>
        <div className="flex flex-col gap-1.5 rounded-md border border-border/35 bg-muted/10 p-2">
            <span className="font-semibold text-[11px]">Shift {index}</span>
            {children}
        </div>
    </div>
);

interface ShiftRoomInit {
    slot_id: string;
    room_type: string;
    formula_type?: string | null;
    active?: boolean;
    recommended?: { operator_id: string; name: string }[];
    current?: { operator_id: string; name: string }[];
    swap_in?: { operator_id: string; name: string }[];
    swap_out?: { operator_id: string; name: string }[];
    moved_out?: { operator: { operator_id: string; name: string }; to_room_type: string; to_team_label?: string | null }[];
    matches?: boolean;
    equivalent?: boolean;
    gap_pct?: number | null;
    efficiency?: number | null;
    team_id?: string | null;
    team_label?: string | null;
}

const shiftRoom = (r: ShiftRoomInit) => ({
    formula_type: null,
    active: true,
    recommended: [],
    current: [],
    swap_in: [],
    swap_out: [],
    moved_out: [],
    matches: false,
    equivalent: false,
    gap_pct: null,
    efficiency: null,
    team_id: null,
    team_label: null,
    ...r,
});

const TEXAS = op("char_102_texas", "Texas");
const EXUSIAI = op("char_103_angel", "Exusiai");
const LAPPLAND = op("char_140_whitew", "Lappland");
const KROOS = op("char_124_kroos", "Kroos");
const VERMEIL = op("char_190_clour", "Vermeil");
const STEWARD = op("char_210_stward", "Steward");
const SCAVENGER = op("char_149_scave", "Scavenger");
const VIGNA = op("char_290_vigna", "Vigna");

/** The common case: two operators to add (green) and one to pull (struck red). */
export const SwapSuggested = () => (
    <Column index={2}>
        <ShiftRoomBlock
            interactive
            room={shiftRoom({
                slot_id: "slot_22",
                room_type: "TRADING",
                recommended: [TEXAS, EXUSIAI, LAPPLAND],
                current: [TEXAS, KROOS],
                swap_in: [EXUSIAI, LAPPLAND],
                swap_out: [KROOS],
                efficiency: 105,
                team_id: "trading-a",
                team_label: "Team A",
            })}
        />
    </Column>
);

/** Your team is inside the leniency band, so it stays put — the system's pick is kept
 *  visible in the muted "rec" row with the gap it would close. */
export const EquivalentToYours = () => (
    <Column index={1}>
        <ShiftRoomBlock
            interactive
            room={shiftRoom({
                slot_id: "slot_12",
                room_type: "MANUFACTURE",
                formula_type: "F_GOLD",
                recommended: [SCAVENGER, VIGNA, STEWARD],
                current: [VERMEIL, STEWARD],
                equivalent: true,
                gap_pct: -2.4,
                efficiency: 128,
                team_id: "factory-b",
                team_label: "Team B",
            })}
        />
    </Column>
);

/** Preset already matches the recommendation (✓), and the Control Center rests this
 *  shift by design (· off). */
export const MatchedAndOffShift = () => (
    <Column index={3}>
        <ShiftRoomBlock
            interactive
            room={shiftRoom({
                slot_id: "slot_13",
                room_type: "MANUFACTURE",
                formula_type: "F_EXP",
                recommended: [VERMEIL, VIGNA],
                current: [VERMEIL, VIGNA],
                matches: true,
                efficiency: 95,
                team_id: "factory-c",
                team_label: "Team C",
            })}
        />
        <ShiftRoomBlock interactive room={shiftRoom({ slot_id: "slot_02", room_type: "CONTROL", active: false, recommended: [], current: [op("char_003_kalts", "Kal'tsit")], team_id: "cc-1", team_label: "Squad 1" })} />
    </Column>
);

/** Preset operators who aren't benched — they relocate to another room this same shift. */
export const RelocatedOperators = () => (
    <Column index={2}>
        <ShiftRoomBlock
            interactive
            room={shiftRoom({
                slot_id: "slot_31",
                room_type: "POWER",
                recommended: [op("char_253_greyy", "Greyy")],
                current: [op("char_143_ghost", "Specter"), op("char_298_susuro", "Sussurro")],
                swap_in: [op("char_253_greyy", "Greyy")],
                moved_out: [
                    { operator: op("char_143_ghost", "Specter"), to_room_type: "MANUFACTURE", to_team_label: "Team B" },
                    { operator: op("char_298_susuro", "Sussurro"), to_room_type: "MEETING", to_team_label: null },
                ],
                efficiency: 15,
                team_id: "power-1",
                team_label: "Squad 1",
            })}
        />
    </Column>
);

/** No preset saved for this room yet, and two operators a morale manager keeps at full
 *  morale every shift (24/7). */
export const SustainedNoPreset = () => (
    <Column index={1}>
        <ShiftRoomBlock
            interactive
            sustained={new Set(["char_4064_mlynar", "char_1028_texas2"])}
            room={shiftRoom({
                slot_id: "slot_23",
                room_type: "TRADING",
                recommended: [op("char_4064_mlynar", "Mlynar"), op("char_1028_texas2", "Texas the Omertosa")],
                swap_in: [op("char_4064_mlynar", "Mlynar"), op("char_1028_texas2", "Texas the Omertosa")],
                efficiency: 82,
                team_id: "trading-b",
                team_label: "Team B",
            })}
        />
    </Column>
);
