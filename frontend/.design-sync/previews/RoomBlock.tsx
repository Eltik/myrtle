import type { CSSProperties, ReactNode } from "react";
import { MiniChip, RoomBlock, RowKicker } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The `right` slot in a peak cell: the room's order-acquisition bonus, tinted with
 *  the same in-game room hue the block's left accent strip uses. */
const Eff = ({ hue, children }: { hue: string; children: ReactNode }) => (
    <span className="font-mono font-semibold text-[10px] tabular-nums" style={{ color: `color-mix(in oklch, ${hue} 60%, var(--foreground))` }}>
        {children}
    </span>
);

/** One block per room, accented with the canonical RIIC hue: Trading Post blue,
 *  Factory gold, Power Plant green, Control Center teal. */
export const ProductionRooms = () => (
    <div className="grid max-w-2xl gap-1.5 sm:grid-cols-2" style={ACCENT}>
        <RoomBlock roomType="TRADING" right={<Eff hue="#0075a9">+105%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_102_texas", "Texas")} />
                <MiniChip op={op("char_103_angel", "Exusiai")} />
                <MiniChip op={op("char_140_whitew", "Lappland")} />
            </div>
            <div className="flex flex-wrap items-center gap-x-1.5 font-mono text-[9px] text-muted-foreground/65">
                <span>25.7k LMD/day*</span>
            </div>
        </RoomBlock>
        <RoomBlock roomType="MANUFACTURE" formula="F_GOLD" right={<Eff hue="#ffd800">+130%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_190_clour", "Vermeil")} />
                <MiniChip op={op("char_149_scave", "Scavenger")} />
                <MiniChip op={op("char_210_stward", "Steward")} />
            </div>
            <div className="flex flex-wrap items-center gap-x-1.5 font-mono text-[9px] text-muted-foreground/65">
                <span>1.6k gold/day</span>
            </div>
        </RoomBlock>
        <RoomBlock roomType="POWER" right={<Eff hue="#8fc31f">+15%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_253_greyy", "Greyy")} />
            </div>
        </RoomBlock>
        <RoomBlock roomType="CONTROL" right={<Eff hue="#005752">+35%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_003_kalts", "Kal'tsit")} />
                <MiniChip op={op("char_128_plosis", "Ptilopsis")} />
                <MiniChip op={op("char_002_amiya", "Amiya")} />
            </div>
            <div className="flex flex-wrap items-center gap-x-1.5 font-mono text-[9px] text-muted-foreground/65">
                <span>global buff</span>
                <span>clue +15% · HR +10%</span>
            </div>
        </RoomBlock>
    </div>
);

/** `badge` carries the cell state marker: a locked synergy squad, or a matched preset. */
export const StateBadges = () => (
    <div className="flex max-w-md flex-col gap-1.5" style={ACCENT}>
        <RoomBlock roomType="MANUFACTURE" formula="F_EXP" badge={<span className="text-amber-500/80">🔒</span>} right={<Eff hue="#ffd800">+95%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_164_nightm", "Nightmare")} />
                <MiniChip op={op("char_290_vigna", "Vigna")} />
                <MiniChip op={op("char_337_utage", "Utage")} />
            </div>
            <div className="flex flex-wrap items-center gap-x-1.5 font-mono text-[9px] text-muted-foreground/65">
                <span>4.2k EXP/day</span>
            </div>
        </RoomBlock>
        <RoomBlock roomType="TRADING" badge={<span className="text-emerald-500/80">· ✓</span>} right={<Eff hue="#0075a9">+95%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_476_blkngt", "Blacknight")} />
                <MiniChip op={op("char_272_strong", "Jaye")} />
            </div>
        </RoomBlock>
        <RoomBlock roomType="HIRE" right={<span className="text-[9px] text-muted-foreground/70">economy</span>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_254_vodfox", "Shamare")} />
            </div>
        </RoomBlock>
    </div>
);

/** `dim` marks a room deliberately unstaffed this shift; the `out`/`moves` rows show
 *  what leaves it. */
export const OffShiftAndMoves = () => (
    <div className="flex max-w-md flex-col gap-1.5" style={ACCENT}>
        <RoomBlock roomType="CONTROL" dim badge={<span className="text-amber-500/80">· off</span>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <span className="text-[10px] text-muted-foreground/50">-</span>
            </div>
        </RoomBlock>
        <RoomBlock roomType="MEETING" right={<Eff hue="#dd653f">+45%</Eff>}>
            <div className="flex flex-wrap items-center gap-0.5">
                <MiniChip op={op("char_4045_heidi", "Heidi")} tone="add" />
                <MiniChip op={op("char_298_susuro", "Sussurro")} />
            </div>
            <div className="flex flex-wrap items-center gap-0.5">
                <RowKicker>out</RowKicker>
                <MiniChip op={op("char_124_kroos", "Kroos")} tone="remove" strike />
            </div>
            <div className="flex flex-wrap items-center gap-0.5">
                <RowKicker>moves</RowKicker>
                <MiniChip op={op("char_143_ghost", "Specter")} dashed suffix={<span className="text-[8px] text-muted-foreground/70">→ Office · Squad 2</span>} />
            </div>
        </RoomBlock>
    </div>
);
