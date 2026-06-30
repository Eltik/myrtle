import { useMemo } from "react";
import { enemyIconURL, type IEnemy } from "#/lib/api/enemies";
import type { ILevel } from "#/lib/api/level";
import { cn } from "#/lib/utils";
import { ENEMY_LEVEL_ACCENT } from "./constants";
import { buildSpawnSchedule } from "./helpers";
import { Kicker, Pill } from "./primitives";
import type { ISpawnRow } from "./types";

function SpawnEnemyIdentity({ row, iconClass, onActivate }: { row: ISpawnRow; iconClass: string; onActivate: () => void }) {
    const accent = ENEMY_LEVEL_ACCENT[row.enemy?.enemyLevel ?? "NORMAL"];
    return (
        <button type="button" aria-label={`Show ${row.enemy?.name ?? row.id} on map`} onClick={onActivate} className="group/enemy flex min-w-0 cursor-pointer items-center gap-2.5 text-left">
            <span className={cn("flex shrink-0 items-center justify-center overflow-hidden rounded border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]", iconClass)} style={{ borderColor: `color-mix(in oklch, ${accent} 40%, var(--border))` }}>
                <img src={enemyIconURL(row.id)} alt="" loading="lazy" className="h-full w-full object-contain" />
            </span>
            <span className="flex min-w-0 flex-col gap-0.5">
                <span className="truncate font-medium font-sans text-[12.5px] text-foreground leading-tight transition-colors group-hover/enemy:text-primary">{row.enemy?.name ?? row.id}</span>
                <span className="font-medium font-mono text-[9px] uppercase leading-none tracking-widest" style={{ color: accent }}>
                    {row.enemy?.enemyLevel ?? row.actionType}
                </span>
            </span>
        </button>
    );
}

function SpawnStat({ label, value, strong }: { label: string; value: string; strong?: boolean }) {
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <dt className="font-medium font-mono text-[8.5px] text-muted-foreground uppercase leading-none tracking-widest">{label}</dt>
            <dd className={cn("font-mono text-[12px] tabular-nums leading-none", strong ? "font-semibold text-foreground" : "text-muted-foreground")}>{value}</dd>
        </div>
    );
}

function SpawnCard({ row, onFocusEnemy }: { row: ISpawnRow; onFocusEnemy: (id: string, time: number) => void }) {
    return (
        <li className="flex flex-col gap-2.5 px-4 py-3">
            <div className="flex items-start justify-between gap-3">
                <SpawnEnemyIdentity iconClass="h-9 w-9" onActivate={() => onFocusEnemy(row.id, row.time)} row={row} />
                <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">#{row.idx + 1}</span>
            </div>
            <dl className="grid grid-cols-3 gap-x-3 gap-y-2.5 rounded-md bg-[color-mix(in_oklch,var(--muted)_30%,transparent)] px-3 py-2.5">
                <SpawnStat label="Wave" value={String(row.wave)} />
                <SpawnStat label="Count" strong value={`×${row.count}`} />
                <SpawnStat label="Start ~" strong value={`${row.time.toFixed(1)}s`} />
                <SpawnStat label="Interval" value={`${row.interval.toFixed(1)}s`} />
                <SpawnStat label="Pre-Delay" value={`${row.preDelay.toFixed(1)}s`} />
            </dl>
        </li>
    );
}

export function SpawnSchedule({ level, enemyData, onFocusEnemy }: { level: ILevel | null; enemyData: Record<string, IEnemy>; onFocusEnemy: (id: string, time: number) => void }) {
    const { rows, hiddenGroups } = useMemo(() => buildSpawnSchedule(level, enemyData), [level, enemyData]);
    if (rows.length === 0) return null;
    return (
        <section className="overflow-hidden rounded-[14px] border border-border bg-card">
            <div className="flex flex-wrap items-center gap-2.5 border-border border-b px-4 py-3">
                <Kicker>Spawn Schedule</Kicker>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{rows.length} groups</span>
                <span className="h-px flex-1 bg-border" />
                {hiddenGroups.length > 0 && (
                    <span className="flex flex-wrap items-center gap-1.5">
                        <span className="font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.12em]">Hidden Routes</span>
                        {hiddenGroups.map((g) => (
                            <Pill key={g}>{g}</Pill>
                        ))}
                    </span>
                )}
            </div>
            <div className="max-h-140 overflow-auto">
                <ul className="divide-y divide-border/40 sm:hidden">
                    {rows.map((r) => (
                        <SpawnCard key={r.idx} onFocusEnemy={onFocusEnemy} row={r} />
                    ))}
                </ul>
                <table className="hidden w-full min-w-150 border-collapse text-left sm:table">
                    <thead className="sticky top-0 z-10 bg-card">
                        <tr className="border-border border-b [&>th]:px-3 [&>th]:py-2.5 [&>th]:font-medium [&>th]:font-mono [&>th]:text-[9.5px] [&>th]:text-muted-foreground [&>th]:uppercase [&>th]:tracking-widest">
                            <th className="w-10 text-right">#</th>
                            <th>Enemy</th>
                            <th className="text-center">Wave</th>
                            <th className="text-right">Count</th>
                            <th className="text-right">Interval</th>
                            <th className="text-right">Pre-Delay</th>
                            <th className="text-right">Start ~</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows.map((r) => (
                            <tr key={r.idx} className="border-border/40 border-b transition-colors last:border-b-0 hover:bg-[color-mix(in_oklch,var(--muted)_25%,transparent)] [&>td]:px-3 [&>td]:py-2">
                                <td className="text-right font-mono text-[11px] text-muted-foreground tabular-nums">{r.idx + 1}</td>
                                <td>
                                    <SpawnEnemyIdentity iconClass="h-8 w-8" onActivate={() => onFocusEnemy(r.id, r.time)} row={r} />
                                </td>
                                <td className="text-center font-mono text-[11.5px] text-muted-foreground tabular-nums">{r.wave}</td>
                                <td className="text-right font-mono font-semibold text-[11.5px] text-foreground tabular-nums">×{r.count}</td>
                                <td className="text-right font-mono text-[11.5px] text-muted-foreground tabular-nums">{r.interval.toFixed(1)}s</td>
                                <td className="text-right font-mono text-[11.5px] text-muted-foreground tabular-nums">{r.preDelay.toFixed(1)}s</td>
                                <td className="text-right font-mono text-[11.5px] text-foreground tabular-nums">{r.time.toFixed(1)}s</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
