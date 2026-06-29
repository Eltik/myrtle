import { Link } from "@tanstack/react-router";
import { enemyIconURL, type IEnemy } from "#/lib/api/enemies";
import { ENEMY_LEVEL_ACCENT } from "./constants";
import { SectionHead } from "./primitives";
import type { IEnemyTally } from "./types";

function EnemyTallyCard({ enemy, id, count }: { enemy: IEnemy | null; id: string; count: number }) {
    const accent = ENEMY_LEVEL_ACCENT[enemy?.enemyLevel ?? "NORMAL"];
    const inner = (
        <div className="group flex h-full items-center gap-3 rounded-[10px] border border-border bg-card p-2.5 transition-colors hover:border-[color-mix(in_oklch,var(--primary)_50%,transparent)]">
            <span className="relative flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-md border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]" style={{ borderColor: `color-mix(in oklch, ${accent} 45%, var(--border))` }}>
                <img src={enemyIconURL(id)} alt={enemy?.name ?? id} loading="lazy" className="h-full w-full object-contain" />
            </span>
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <span className="truncate font-sans font-semibold text-[12.5px] text-foreground leading-tight">{enemy?.name ?? id}</span>
                <span className="font-medium font-mono text-[10px] uppercase leading-none tracking-widest" style={{ color: accent }}>
                    {enemy?.enemyLevel ?? "UNKNOWN"}
                </span>
            </div>
            {count > 0 && <span className="shrink-0 rounded-full bg-[color-mix(in_oklch,var(--muted)_55%,transparent)] px-2 py-1 font-mono font-semibold text-[11px] text-foreground tabular-nums leading-none">×{count}</span>}
        </div>
    );
    return enemy ? (
        <Link to="/enemies/$id" params={{ id }} className="block">
            {inner}
        </Link>
    ) : (
        inner
    );
}

export function EnemiesSection({ tally }: { tally: IEnemyTally[] }) {
    if (tally.length === 0) return null;
    return (
        <section>
            <SectionHead>Enemies · {tally.length}</SectionHead>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-3">
                {tally.map(({ enemy, id, count }) => (
                    <EnemyTallyCard key={id} count={count} enemy={enemy} id={id} />
                ))}
            </div>
        </section>
    );
}
