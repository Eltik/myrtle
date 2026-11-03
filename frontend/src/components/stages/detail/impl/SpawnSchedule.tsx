import { useMemo } from "react";
import { enemyIconURL, type IEnemy } from "#/lib/api/enemies";
import type { ILevel } from "#/lib/api/level";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { SparseRecord } from "#/lib/records";
import { cn } from "#/lib/utils";
import { ENEMY_LEVEL_ACCENT, ENEMY_LEVEL_INK, ENEMY_LEVEL_LABEL_KEY } from "./constants";
import type { messages as detailConstantsMessages } from "./constants.messages";
import type { messages as enemiesSectionMessages } from "./EnemiesSection.messages";
import { buildSpawnSchedule } from "./helpers";
import { Pill } from "./primitives";
import type { messages } from "./SpawnSchedule.messages";
import type { ISpawnRow } from "./types";

function SpawnEnemyIdentity({ row, iconClass, onActivate }: { row: ISpawnRow; iconClass: string; onActivate: () => void }) {
    const tEnemies: TypedT<typeof enemiesSectionMessages> = useT("stages");
    const tConst: TypedT<typeof detailConstantsMessages> = useT("stages");
    const accent = ENEMY_LEVEL_ACCENT[row.enemy?.enemyLevel ?? "NORMAL"];
    const ink = ENEMY_LEVEL_INK[row.enemy?.enemyLevel ?? "NORMAL"];
    return (
        <button type="button" aria-label={tEnemies("enemies.showOnMap", { name: row.enemy?.name ?? row.id })} onClick={onActivate} className="group/enemy flex min-w-0 cursor-pointer items-center gap-2.5 text-left">
            <span className={cn("flex shrink-0 items-center justify-center overflow-hidden rounded border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]", iconClass)} style={{ borderColor: `color-mix(in oklch, ${accent} 40%, var(--border))` }}>
                <img src={enemyIconURL(row.id)} alt="" loading="lazy" className="h-full w-full object-contain" />
            </span>
            <span className="flex min-w-0 flex-col gap-0.5">
                <span className="truncate font-medium font-sans text-[12.5px] text-foreground leading-tight transition-colors group-hover/enemy:text-primary">{row.enemy?.name ?? row.id}</span>
                <span className="font-medium font-mono text-[10px] leading-none" style={{ color: ink }}>
                    {tConst(row.enemy ? ENEMY_LEVEL_LABEL_KEY[row.enemy.enemyLevel] : "enemyLevel.UNKNOWN")}
                </span>
            </span>
        </button>
    );
}

function SpawnStat({ label, value, strong }: { label: string; value: string; strong?: boolean }) {
    return (
        <div className="flex min-w-0 flex-col gap-1">
            <dt className="font-medium font-sans text-[11.5px] text-muted-foreground leading-none">{label}</dt>
            <dd className={cn("font-mono text-[12px] tabular-nums leading-none", strong ? "font-semibold text-foreground" : "text-muted-foreground")}>{value}</dd>
        </div>
    );
}

function SpawnCard({ row, onFocusEnemy }: { row: ISpawnRow; onFocusEnemy: (id: string, time: number) => void }) {
    const t: TypedT<typeof messages> = useT("stages");
    return (
        <li className="flex flex-col gap-2.5 px-4 py-3">
            <div className="flex items-start justify-between gap-3">
                <SpawnEnemyIdentity iconClass="h-9 w-9" onActivate={() => onFocusEnemy(row.id, row.time)} row={row} />
                <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">#{row.idx + 1}</span>
            </div>
            <dl className="grid grid-cols-2 gap-x-3 gap-y-2.5 rounded-md bg-[color-mix(in_oklch,var(--muted)_30%,transparent)] px-3 py-2.5 max-sm:*:last:odd:col-span-2 sm:grid-cols-3">
                <SpawnStat label={t("spawns.col.wave")} value={String(row.wave)} />
                <SpawnStat label={t("spawns.col.count")} strong value={`×${row.count}`} />
                <SpawnStat label={t("spawns.col.start")} strong value={`${row.time.toFixed(1)}s`} />
                <SpawnStat label={t("spawns.col.interval")} value={`${row.interval.toFixed(1)}s`} />
                <SpawnStat label={t("spawns.col.preDelay")} value={`${row.preDelay.toFixed(1)}s`} />
            </dl>
        </li>
    );
}

export function SpawnSchedule({ level, enemyData, onFocusEnemy }: { level: ILevel | null; enemyData: SparseRecord<IEnemy>; onFocusEnemy: (id: string, time: number) => void }) {
    const t: TypedT<typeof messages> = useT("stages");
    const { rows, hiddenGroups } = useMemo(() => buildSpawnSchedule(level, enemyData), [level, enemyData]);
    if (rows.length === 0) return null;
    return (
        <section className="overflow-hidden rounded-2xl border border-border bg-card">
            <div className="flex flex-wrap items-center gap-2.5 border-border border-b px-4 py-3">
                <h2 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-none">{t("spawns.title")}</h2>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("spawns.groups", { count: rows.length })}</span>
                <span className="h-px flex-1 bg-border" />
                {hiddenGroups.length > 0 && (
                    <span className="flex flex-wrap items-center gap-1.5">
                        <span className="font-medium font-sans text-[11.5px] text-muted-foreground">{t("spawns.hiddenRoutes")}</span>
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
                        <tr className="border-border border-b [&>th]:px-3 [&>th]:py-2.5 [&>th]:font-medium [&>th]:font-sans [&>th]:text-[11.5px] [&>th]:text-muted-foreground">
                            <th className="w-10 text-right">#</th>
                            <th>{t("spawns.col.enemy")}</th>
                            <th className="text-center">{t("spawns.col.wave")}</th>
                            <th className="text-right">{t("spawns.col.count")}</th>
                            <th className="text-right">{t("spawns.col.interval")}</th>
                            <th className="text-right">{t("spawns.col.preDelay")}</th>
                            <th className="text-right">{t("spawns.col.start")}</th>
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
