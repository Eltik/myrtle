import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IMoraleTimeline } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_KICKER } from "../shared";
import { TEXT_TINY } from "./parts";
import { roomAccent, roomLabel } from "./roomColors";

const MORALE_MAX = 24;

/** Inline sparkline of one operator's simulated morale over the week. */
function Sparkline({ samples }: { samples: number[] }) {
    const w = 96;
    const h = 26;
    const pad = 1.5;
    if (samples.length < 2) return <div style={{ width: w, height: h }} />;
    const step = (w - pad * 2) / (samples.length - 1);
    const y = (m: number) => pad + (1 - Math.max(0, Math.min(MORALE_MAX, m)) / MORALE_MAX) * (h - pad * 2);
    const points = samples.map((m, i) => `${(pad + i * step).toFixed(1)},${y(m).toFixed(1)}`).join(" ");
    return (
        <svg width={w} height={h} viewBox={`0 0 ${w} ${h}`} className="shrink-0" aria-hidden>
            <polyline points={points} fill="none" stroke="var(--color-red-500, #ef4444)" strokeWidth="1.5" strokeLinejoin="round" strokeLinecap="round" />
        </svg>
    );
}

/**
 * "Morale over time": one card per simulated operator - avatar, home room, a
 * week-long sparkline of their bar, and where it ends. The data is the same
 * simulation the sustainability verdict comes from, so a sawtooth here IS the
 * rotation rhythm, and a decaying line is the leak the banner warns about.
 */
export function MoraleOverTime({ timeline }: { timeline: IMoraleTimeline[] }) {
    if (timeline.length === 0) return null;
    return (
        <div className="flex flex-col gap-1.5">
            <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Morale over time</span>
            <div className="grid max-h-72 grid-cols-2 gap-1.5 overflow-y-auto pr-1 sm:grid-cols-3 lg:grid-cols-4">
                {timeline.map((row) => {
                    const a = roomAccent(row.room_type);
                    const low = row.end < MORALE_MAX / 2;
                    return (
                        <div key={row.operator.operator_id} className="flex items-center gap-2 rounded-md border border-border/35 bg-muted/10 px-2 py-1.5">
                            <span className="relative size-7 shrink-0 overflow-hidden rounded-sm border border-border/40">
                                <OperatorAvatar charId={row.operator.operator_id} name={row.operator.name} />
                            </span>
                            <div className="flex min-w-0 flex-1 flex-col">
                                <span className={cn("truncate font-medium", TEXT_TINY)}>{row.operator.name}</span>
                                <span className={cn("truncate", TEXT_TINY)} style={{ color: a.text }}>
                                    {roomLabel(row.room_type)}
                                </span>
                                <Sparkline samples={row.samples} />
                            </div>
                            <span className={cn("shrink-0 font-mono font-semibold tabular-nums", TEXT_TINY, low ? "text-red-500/90" : "text-foreground/80")}>
                                {row.end.toFixed(1)}
                                <span className="text-muted-foreground/60">/{MORALE_MAX}</span>
                            </span>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
