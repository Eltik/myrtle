import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IMoraleTimeline } from "#/lib/api/user";
import { type Catalog, roomLabel } from "#/lib/base/catalog";
import { roomAccent } from "#/lib/base/room-colors";
import { cn } from "#/lib/utils";

const MORALE_MAX = 24;

function Sparkline({ samples }: { samples: number[] }) {
    const w = 96;
    const h = 26;
    const pad = 1.5;
    if (samples.length < 2) return <div style={{ height: h, width: w }} />;

    const step = (w - pad * 2) / (samples.length - 1);
    const y = (morale: number) => pad + (1 - Math.max(0, Math.min(MORALE_MAX, morale)) / MORALE_MAX) * (h - pad * 2);
    const points = samples.map((morale, index) => `${(pad + index * step).toFixed(1)},${y(morale).toFixed(1)}`).join(" ");

    return (
        <svg className="shrink-0" height={h} role="img" viewBox={`0 0 ${w} ${h}`} width={w}>
            <title>Morale across the simulated week</title>
            <polyline fill="none" points={points} stroke="var(--color-red-500, #ef4444)" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" />
        </svg>
    );
}

export function MoraleTimeline({ catalog, timeline }: { catalog: Catalog; timeline: IMoraleTimeline[] }) {
    return (
        <div className="grid max-h-72 grid-cols-1 gap-1.5 overflow-y-auto pr-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {timeline.map((row) => {
                const accent = roomAccent(row.room_type);
                const low = row.end < MORALE_MAX / 2;

                return (
                    <div className="flex items-center gap-2 rounded-md border border-border/35 bg-muted/10 px-2 py-1.5" key={`${row.slot_id}:${row.operator.operator_id}`}>
                        <span className="relative size-7 shrink-0 overflow-hidden rounded-sm border border-border/40">
                            <OperatorAvatar charId={row.operator.operator_id} name={row.operator.name} />
                        </span>
                        <div className="flex min-w-0 flex-1 flex-col">
                            <span className="truncate font-medium text-[10px]">{row.operator.name}</span>
                            <span className="truncate text-[9px]" style={{ color: accent.text }}>
                                {roomLabel(row.room_type, catalog)}
                            </span>
                            <Sparkline samples={row.samples} />
                        </div>
                        <span className={cn("shrink-0 font-mono font-semibold text-[11px] tabular-nums", low ? "text-red-500/90" : "text-foreground/80")}>
                            {row.end.toFixed(1)}
                            <span className="text-[9px] text-muted-foreground/60">/{MORALE_MAX}</span>
                        </span>
                    </div>
                );
            })}
        </div>
    );
}
