import { ChartColumn } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { enemyIconURL, type IEnemy } from "#/lib/api/enemies";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ENEMY_LEVEL_ACCENT, ENEMY_LEVEL_INK, ENEMY_LEVEL_LABEL_KEY } from "./constants";
import type { messages as detailConstantsMessages } from "./constants.messages";
import type { messages } from "./EnemiesSection.messages";
import { EnemyStatsCard } from "./EnemyStatsCard";
import { SectionHead } from "./primitives";
import type { IEnemyTally, IStageEnemyStats } from "./types";

/** Resolves an enemy's stage-effective stats, the same numbers the map's hover card shows. */
type StatsForFn = (id: string, enemy: IEnemy | null) => IStageEnemyStats | null;

function EnemyTallyCard({ enemy, id, count, onFocusEnemy, statsFor }: { enemy: IEnemy | null; id: string; count: number; onFocusEnemy: (id: string) => void; statsFor: StatsForFn }) {
    const t: TypedT<typeof messages> = useT("stages");
    const tConst: TypedT<typeof detailConstantsMessages> = useT("stages");
    const accent = ENEMY_LEVEL_ACCENT[enemy?.enemyLevel ?? "NORMAL"];
    const ink = ENEMY_LEVEL_INK[enemy?.enemyLevel ?? "NORMAL"];
    const name = enemy?.name ?? id;
    return (
        <div className="flex h-full items-center gap-2 rounded-2xl border border-border bg-card p-1.5 pe-2.5 transition-colors has-[[data-enemy-focus]:hover]:border-[color-mix(in_oklch,var(--primary)_50%,transparent)]">
            <button type="button" data-enemy-focus aria-label={t("enemies.showOnMap", { name })} onClick={() => onFocusEnemy(id)} className="group/enemy flex min-w-0 flex-1 cursor-pointer items-center gap-3 rounded-xl p-1 text-left focus-visible:ring-2 focus-visible:ring-ring">
                <span className="relative flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-md border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]" style={{ borderColor: `color-mix(in oklch, ${accent} 45%, var(--border))` }}>
                    <img src={enemyIconURL(id)} alt={name} loading="lazy" width={44} height={44} className="h-full w-full object-contain" />
                </span>
                <span className="flex min-w-0 flex-1 flex-col gap-1">
                    <span className="line-clamp-2 break-words font-sans font-semibold text-[12.5px] text-foreground leading-tight transition-colors group-hover/enemy:text-primary">{name}</span>
                    <span className="whitespace-nowrap font-medium font-mono text-[10px] leading-none" style={{ color: ink }}>
                        {tConst(enemy ? ENEMY_LEVEL_LABEL_KEY[enemy.enemyLevel] : "enemyLevel.UNKNOWN")}
                    </span>
                </span>
                {count > 0 && <span className="shrink-0 rounded-full bg-[color-mix(in_oklch,var(--muted)_55%,transparent)] px-2 py-1 font-mono font-semibold text-[11px] text-foreground tabular-nums leading-none">×{count}</span>}
            </button>
            {/* The map shows these stats only on pointer hover; this is the tap and keyboard path to the same card. */}
            <Popover>
                <PopoverTrigger render={<Button aria-label={t("enemies.stats.aria", { name })} size="xs" variant="outline" className="shrink-0" />}>
                    <ChartColumn aria-hidden="true" />
                    {t("enemies.stats")}
                </PopoverTrigger>
                <PopoverPopup align="end">
                    <EnemyStatsCard enemy={enemy} id={id} stats={statsFor(id, enemy)} />
                </PopoverPopup>
            </Popover>
        </div>
    );
}

export function EnemiesSection({ tally, onFocusEnemy, statsFor }: { tally: IEnemyTally[]; onFocusEnemy: (id: string) => void; statsFor: StatsForFn }) {
    const t: TypedT<typeof messages> = useT("stages");
    if (tally.length === 0) return null;
    return (
        <section>
            <SectionHead>{t("enemies.title", { count: tally.length })}</SectionHead>
            {/* Two columns at most: the section sits in half the page from lg, and three cards there cut a name to "Ex…" and broke "Normal". */}
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                {tally.map(({ enemy, id, count }) => (
                    <EnemyTallyCard key={id} count={count} enemy={enemy} id={id} onFocusEnemy={onFocusEnemy} statsFor={statsFor} />
                ))}
            </div>
        </section>
    );
}
