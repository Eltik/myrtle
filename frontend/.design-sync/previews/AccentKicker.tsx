import type { CSSProperties } from "react";
import { AccentKicker, MiniChip, RoomBlock } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The kicker heading the Base panel's current-vs-optimal block. */
export const SectionHeading = () => (
    <div className="flex max-w-md flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3" style={ACCENT}>
        <div className="flex flex-wrap items-center justify-between gap-2">
            <AccentKicker>Current vs optimal</AccentKicker>
            <span className="font-mono text-[10px] text-muted-foreground tabular-nums">+13.2k value/day</span>
        </div>
        <p className="text-[10.5px] text-muted-foreground leading-snug">Restationing four operators lifts sustained output by 26 percentage points.</p>
    </div>
);

/** Numbering the overlapping rotation sets, as `RotationSection` composes it. */
export const RotationSetRows = () => (
    <div className="grid max-w-2xl gap-1.5 sm:grid-cols-2" style={ACCENT}>
        <div className="flex flex-col gap-1 rounded-md border border-border/35 bg-muted/15 px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
                <AccentKicker>Set 1</AccentKicker>
                <span className="truncate font-mono text-[10px] text-muted-foreground/60 tabular-nums">resting: Scavenger</span>
            </div>
            <div className="flex flex-col gap-1">
                <RoomBlock roomType="TRADING">
                    <div className="flex flex-wrap items-center gap-0.5">
                        <MiniChip op={op("char_102_texas", "Texas")} />
                        <MiniChip op={op("char_103_angel", "Exusiai")} />
                    </div>
                </RoomBlock>
                <RoomBlock roomType="MANUFACTURE" formula="F_GOLD">
                    <div className="flex flex-wrap items-center gap-0.5">
                        <MiniChip op={op("char_190_clour", "Vermeil")} />
                        <MiniChip op={op("char_290_vigna", "Vigna")} />
                    </div>
                </RoomBlock>
            </div>
        </div>
        <div className="flex flex-col gap-1 rounded-md border border-border/35 bg-muted/15 px-2 py-1.5">
            <div className="flex items-center justify-between gap-2">
                <AccentKicker>Set 2</AccentKicker>
                <span className="truncate font-mono text-[10px] text-muted-foreground/60 tabular-nums">resting: Texas, Vermeil</span>
            </div>
            <div className="flex flex-col gap-1">
                <RoomBlock roomType="TRADING">
                    <div className="flex flex-wrap items-center gap-0.5">
                        <MiniChip op={op("char_140_whitew", "Lappland")} />
                        <MiniChip op={op("char_103_angel", "Exusiai")} />
                    </div>
                </RoomBlock>
                <RoomBlock roomType="MANUFACTURE" formula="F_GOLD">
                    <div className="flex flex-wrap items-center gap-0.5">
                        <MiniChip op={op("char_149_scave", "Scavenger")} />
                        <MiniChip op={op("char_290_vigna", "Vigna")} />
                    </div>
                </RoomBlock>
            </div>
        </div>
    </div>
);

/** The accent is a CSS variable, so the same kicker takes each subscore's own hue. */
export const AcrossSubscores = () => (
    <div className="flex max-w-md flex-col gap-2">
        <div className="rounded-md border border-border/40 bg-muted/10 p-3" style={{ "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties}>
            <AccentKicker>Base · sustained rotation</AccentKicker>
        </div>
        <div className="rounded-md border border-border/40 bg-muted/10 p-3" style={{ "--imp-accent": "oklch(0.74 0.17 75)" } as CSSProperties}>
            <AccentKicker>Operators · below milestone</AccentKicker>
        </div>
        <div className="rounded-md border border-border/40 bg-muted/10 p-3" style={{ "--imp-accent": "oklch(0.62 0.20 255)" } as CSSProperties}>
            <AccentKicker>Stages · unclaimed first-clears</AccentKicker>
        </div>
    </div>
);
