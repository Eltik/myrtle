import type { CSSProperties } from "react";
import { MiniChip, RoomBlock, RowKicker } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** "out" — the operators to pull from this shift, struck through in red. */
export const OutRow = () => (
    <div className="max-w-xs" style={ACCENT}>
        <RoomBlock roomType="TRADING" right={<span className="font-mono font-semibold text-[10px] tabular-nums" style={{ color: "color-mix(in oklch, #0075a9 60%, var(--foreground))" }}>+105%</span>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_102_texas", "Texas")} />
                <MiniChip op={op("char_103_angel", "Exusiai")} tone="add" />
            </div>
            <div className="flex flex-wrap items-center gap-0.5">
                <RowKicker>out</RowKicker>
                <MiniChip op={op("char_124_kroos", "Kroos")} tone="remove" strike />
            </div>
        </RoomBlock>
    </div>
);

/** "rec" — an ≈-yours cell keeps your team on top and shows the system's pick muted below. */
export const RecRow = () => (
    <div className="max-w-xs" style={ACCENT}>
        <RoomBlock roomType="MANUFACTURE" formula="F_GOLD" badge={<span className="text-emerald-500/80">· ≈ yours (−2.4%)</span>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_190_clour", "Vermeil")} />
                <MiniChip op={op("char_210_stward", "Steward")} />
            </div>
            <div className="flex flex-wrap items-center gap-0.5 opacity-70">
                <RowKicker>rec</RowKicker>
                <MiniChip op={op("char_149_scave", "Scavenger")} dashed />
                <MiniChip op={op("char_290_vigna", "Vigna")} dashed />
            </div>
        </RoomBlock>
    </div>
);

/** "moves" — preset operators who aren't benched, they relocate within the same shift. */
export const MovesRow = () => (
    <div className="max-w-xs" style={ACCENT}>
        <RoomBlock roomType="POWER" right={<span className="font-mono font-semibold text-[10px] tabular-nums" style={{ color: "color-mix(in oklch, #8fc31f 60%, var(--foreground))" }}>+15%</span>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_253_greyy", "Greyy")} tone="add" />
            </div>
            <div className="flex flex-wrap items-center gap-0.5">
                <RowKicker>moves</RowKicker>
                <MiniChip op={op("char_143_ghost", "Specter")} dashed suffix={<span className="text-[8px] text-muted-foreground/70">→ Factory · Team B</span>} />
                <MiniChip op={op("char_298_susuro", "Sussurro")} dashed suffix={<span className="text-[8px] text-muted-foreground/70">→ Reception</span>} />
            </div>
        </RoomBlock>
    </div>
);
