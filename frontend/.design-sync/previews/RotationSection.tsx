import type { CSSProperties, ReactNode } from "react";
import { RotationSection } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });
const member = (operator: { operator_id: string; name: string }, lasts_hours: number | null) => ({ operator, lasts_hours });

/** The collapsible body the Base plan dialog drops this section into. */
const Section = ({ count, children }: { count: string; children: ReactNode }) => (
    <div className="flex max-w-3xl flex-col gap-3" style={ACCENT}>
        <div className="flex items-center justify-between gap-2 border-border/40 border-b pb-1.5">
            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em]" style={{ color: "color-mix(in oklch, var(--imp-accent) 60%, var(--foreground))" }}>
                Sustained 24/7 rotation
            </span>
            <span className="rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">{count}</span>
        </div>
        <div className="flex flex-col gap-3 pt-0.5">{children}</div>
    </div>
);

const ROOMS = [
    {
        slot_id: "slot_22",
        room_type: "TRADING",
        members: [member(op("char_102_texas", "Texas"), 8), member(op("char_103_angel", "Exusiai"), 13), member(op("char_140_whitew", "Lappland"), null)],
        backup: op("char_476_blkngt", "Blacknight"),
    },
    {
        slot_id: "slot_12",
        room_type: "MANUFACTURE",
        members: [member(op("char_190_clour", "Vermeil"), 6), member(op("char_149_scave", "Scavenger"), 11), member(op("char_210_stward", "Steward"), null)],
        backup: op("char_290_vigna", "Vigna"),
    },
    {
        slot_id: "slot_13",
        room_type: "MANUFACTURE",
        members: [member(op("char_164_nightm", "Nightmare"), 7), member(op("char_143_ghost", "Specter"), 12)],
        backup: op("char_298_susuro", "Sussurro"),
    },
    {
        slot_id: "slot_31",
        room_type: "POWER",
        members: [member(op("char_253_greyy", "Greyy"), null)],
        backup: null,
    },
];

const SETS = [
    {
        rooms: [
            { slot_id: "slot_22", room_type: "TRADING", working: [op("char_103_angel", "Exusiai"), op("char_140_whitew", "Lappland"), op("char_476_blkngt", "Blacknight")], resting: op("char_102_texas", "Texas") },
            { slot_id: "slot_12", room_type: "MANUFACTURE", working: [op("char_149_scave", "Scavenger"), op("char_210_stward", "Steward"), op("char_290_vigna", "Vigna")], resting: op("char_190_clour", "Vermeil") },
        ],
    },
    {
        rooms: [
            { slot_id: "slot_22", room_type: "TRADING", working: [op("char_102_texas", "Texas"), op("char_140_whitew", "Lappland"), op("char_476_blkngt", "Blacknight")], resting: op("char_103_angel", "Exusiai") },
            { slot_id: "slot_12", room_type: "MANUFACTURE", working: [op("char_190_clour", "Vermeil"), op("char_210_stward", "Steward"), op("char_290_vigna", "Vigna")], resting: op("char_149_scave", "Scavenger") },
        ],
    },
];

/** The full plan: per-room swap order, the shared bench, and the overlapping sets. */
export const StaggeredRotation = () => (
    <Section count="+96.4% sustained">
        <RotationSection rotation={{ rooms: ROOMS, shared_bench: [op("char_263_skadi", "Skadi"), op("char_4045_heidi", "Heidi")], sets: SETS, sustained_efficiency: 96.4 }} />
    </Section>
);

/** A roster deep enough to bench a dedicated backup per room, so no shared bench
 *  and no set cycling is needed. */
export const PerRoomBackupsOnly = () => (
    <Section count="+88.1% sustained">
        <RotationSection rotation={{ rooms: ROOMS, shared_bench: [], sets: undefined, sustained_efficiency: 88.1 }} />
    </Section>
);

/** A thinner roster: two rooms, one shared bench seat covering both. */
export const SharedBench = () => (
    <Section count="+52.0% sustained">
        <RotationSection
            rotation={{
                rooms: [
                    { slot_id: "slot_22", room_type: "TRADING", members: [member(op("char_102_texas", "Texas"), 8), member(op("char_124_kroos", "Kroos"), 14)], backup: null },
                    { slot_id: "slot_12", room_type: "MANUFACTURE", members: [member(op("char_210_stward", "Steward"), null)], backup: null },
                ],
                shared_bench: [op("char_298_susuro", "Sussurro")],
                sets: undefined,
                sustained_efficiency: 52,
            }}
        />
    </Section>
);
