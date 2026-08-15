import type { CSSProperties } from "react";
import { MiniChip, RowKicker } from "frontend";

// Base subscore accent (SCORE_PALETTE.base) — several base-plan parts read `--imp-accent`.
const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** Green = add to your preset, red = pull out, plain = keep. */
export const PresetDiff = () => (
    <div className="flex flex-col gap-2" style={ACCENT}>
        <div className="flex flex-wrap items-center gap-1">
            <RowKicker>rec</RowKicker>
            <MiniChip op={op("char_102_texas", "Texas")} />
            <MiniChip op={op("char_103_angel", "Exusiai")} tone="add" />
            <MiniChip op={op("char_140_whitew", "Lappland")} tone="add" />
        </div>
        <div className="flex flex-wrap items-center gap-1">
            <RowKicker>out</RowKicker>
            <MiniChip op={op("char_124_kroos", "Kroos")} tone="remove" strike />
            <MiniChip op={op("char_476_blkngt", "Blacknight")} tone="remove" strike />
        </div>
    </div>
);

/** Rotation markers: ⚡ swap-first lead and the "~6h before morale runs low" suffix. */
export const RotationMarkers = () => (
    <div className="flex flex-wrap items-center gap-1" style={ACCENT}>
        <MiniChip op={op("char_190_clour", "Vermeil")} lead={<span className="text-[10px] text-amber-500/90">⚡</span>} suffix={<span className="font-mono text-[9px] text-muted-foreground/65 tabular-nums">~6h</span>} />
        <MiniChip op={op("char_149_scave", "Scavenger")} suffix={<span className="font-mono text-[9px] text-muted-foreground/65 tabular-nums">~11h</span>} />
        <MiniChip op={op("char_210_stward", "Steward")} suffix={<span className="font-mono text-[9px] text-muted-foreground/65 tabular-nums">∞</span>} />
    </div>
);

/** Dashed = a bench operator held in reserve, not stationed this shift. */
export const BenchAndBackup = () => (
    <div className="flex flex-wrap items-center gap-1" style={ACCENT}>
        <MiniChip op={op("char_164_nightm", "Nightmare")} />
        <span className="px-0.5 text-[10px]" style={{ color: "color-mix(in oklch, var(--imp-accent) 55%, var(--muted-foreground))" }}>
            ⇄
        </span>
        <MiniChip op={op("char_290_vigna", "Vigna")} dashed suffix={<span className="font-mono text-[9px] text-muted-foreground/55">backup</span>} />
        <MiniChip op={op("char_263_skadi", "Skadi")} dashed suffix={<span className="font-mono text-[9px] text-muted-foreground/55">bench</span>} />
    </div>
);

/** Suffixes carrying plan metadata: the 24/7 morale badge and a station-here arrow. */
export const AnnotatedSuffixes = () => (
    <div className="flex flex-wrap items-center gap-1" style={ACCENT}>
        <MiniChip op={op("char_4064_mlynar", "Mlynar")} suffix={<span className="rounded-sm bg-amber-500/20 px-0.5 font-semibold text-[8px] text-amber-500/90 leading-none">24/7</span>} />
        <MiniChip
            op={op("char_4045_heidi", "Heidi")}
            suffix={
                <span className="font-mono text-[9px]" style={{ color: "color-mix(in oklch, #dd653f 72%, var(--foreground))" }}>
                    → Reception
                </span>
            }
        />
        <MiniChip
            op={op("char_1028_texas2", "Texas the Omertosa")}
            tone="add"
            suffix={
                <span className="font-mono text-[9px]">
                    <span style={{ color: "color-mix(in oklch, #0075a9 60%, var(--foreground))" }}>+38%</span> <span className="text-muted-foreground/60">+22%·24/7</span>
                </span>
            }
        />
    </div>
);
