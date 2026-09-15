import { enemyIconURL, type IEnemy } from "#/lib/api/enemies";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ENEMY_LEVEL_ACCENT } from "./constants";
import type { messages } from "./EnemyStatsCard.messages";
import type { IStageEnemyStats } from "./types";

function StatCell({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex flex-col gap-1 rounded-md border border-border bg-[color-mix(in_oklch,var(--muted)_30%,transparent)] px-2 py-1.5">
            <span className="font-medium font-mono text-[8px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{label}</span>
            <span className="font-mono font-semibold text-[12.5px] text-foreground tabular-nums leading-none">{value}</span>
        </div>
    );
}

export function EnemyStatsCard({ id, enemy, stats }: { id: string; enemy: IEnemy | null; stats: IStageEnemyStats | null }) {
    const t: TypedT<typeof messages> = useT("stages");
    const f = useFormatters();
    const level = enemy?.enemyLevel ?? "NORMAL";
    const accent = ENEMY_LEVEL_ACCENT[level];
    return (
        <div className="flex w-60 flex-col gap-2.5">
            <div className="flex items-center gap-2.5">
                <span className="flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]" style={{ borderColor: `color-mix(in oklch, ${accent} 40%, var(--border))` }}>
                    <img src={enemyIconURL(id)} alt="" loading="lazy" className="h-full w-full object-contain" />
                </span>
                <div className="flex min-w-0 flex-col gap-1">
                    <span className="truncate font-sans font-semibold text-[13px] text-foreground leading-tight">{enemy?.name ?? id}</span>
                    <span className="flex items-center gap-1.5 font-medium font-mono text-[8.5px] uppercase leading-none tracking-[0.14em]">
                        <span style={{ color: accent }}>{level}</span>
                        {stats && stats.phaseCount > 1 && <span className="text-muted-foreground">· L{stats.levelIndex}</span>}
                    </span>
                </div>
            </div>

            {stats ? (
                <div className="grid grid-cols-3 gap-1.5">
                    <StatCell label={t("enemyStats.hp")} value={f.number(stats.maxHp)} />
                    <StatCell label={t("enemyStats.atk")} value={f.number(stats.atk)} />
                    <StatCell label={t("enemyStats.def")} value={f.number(stats.def)} />
                    <StatCell label={t("enemyStats.res")} value={`${stats.res}%`} />
                    <StatCell label={t("enemyStats.spd")} value={stats.moveSpeed.toFixed(2)} />
                    <StatCell label={t("enemyStats.atkTime")} value={`${stats.attackInterval.toFixed(2)}s`} />
                </div>
            ) : (
                <p className="m-0 font-sans text-[11px] text-muted-foreground leading-snug">{t("enemyStats.none")}</p>
            )}
        </div>
    );
}
