import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { env } from "#/env";
import { APPLY_WAY_DISPLAY, LIST_GRID_COLS } from "../constants";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";
import type { IEnemyStatMaxByLevel, IEnemyView } from "../types";
import { DamageDots, LevelBadge } from "./atoms";
import { EnemyPlaceholder } from "./EnemyPlaceholder";

interface IEnemyCardListProps {
    enemy: IEnemyView;
    statMax: IEnemyStatMaxByLevel;
}

export function EnemyCardList({ enemy, statMax }: IEnemyCardListProps) {
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait ? `${env.VITE_BACKEND_URL ?? ""}/api/assets${enemy.portrait}` : undefined;
    const dmgLabel = enemy.damageType.length ? enemy.damageType.map((t) => DAMAGE_TOKENS[t]?.label ?? t).join(" · ") : "-";
    const tierMaxHp = statMax[enemy.enemyLevel].hp;
    const hpPct = tierMaxHp > 0 ? Math.min(100, (enemy.flatStats.maxHp / tierMaxHp) * 100) : 0;

    return (
        <Link
            to="/enemies/$id"
            params={{ id: enemy.enemyId }}
            className="enemy-row group relative grid w-full cursor-pointer items-center gap-3.5 rounded-lg border border-transparent bg-card/50 px-3.5 py-2.5 text-left transition-colors hover:border-border hover:bg-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring max-[900px]:grid-cols-[48px_1fr_auto]! max-[900px]:[&>:nth-child(4)]:hidden max-[900px]:[&>:nth-child(5)]:hidden"
            style={{ gridTemplateColumns: LIST_GRID_COLS }}
            aria-label={`${enemy.name} (${enemy.enemyIndex})`}
        >
            <span
                aria-hidden="true"
                className="absolute top-1/2 left-0 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60 transition-opacity group-hover:opacity-100"
                style={{
                    background: enemy.enemyLevel === "NORMAL" ? "transparent" : tok.accent,
                }}
            />

            <div className="relative h-12 w-12 overflow-hidden rounded-md border border-[color-mix(in_oklch,var(--border)_60%,transparent)] bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]">
                {hasPortrait && portraitSrc ? (
                    <img src={portraitSrc} alt={`${enemy.name} portrait`} loading="lazy" decoding="async" onError={() => setImgError(true)} className="absolute inset-0 h-full w-full object-contain transition-transform duration-200 ease-out group-hover:scale-110" />
                ) : (
                    <EnemyPlaceholder className="absolute inset-0 h-full w-full p-1.5" />
                )}
            </div>

            <div className="flex min-w-0 flex-col gap-0.75">
                <span className="truncate font-sans font-semibold text-[13px] text-foreground uppercase leading-[1.1] tracking-[0.03em]">{enemy.name}</span>
                <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">
                    {enemy.enemyIndex}
                    {enemy.race ? ` · ${enemy.race}` : ""}
                </span>
            </div>

            <div className="justify-self-start">{enemy.enemyLevel === "NORMAL" ? <span className="font-medium font-sans text-[11.5px] text-muted-foreground">Normal</span> : <LevelBadge level={enemy.enemyLevel} />}</div>

            <div className="inline-flex items-center gap-2">
                <DamageDots types={enemy.damageType} size={7} />
                <span className="truncate font-sans text-[12.5px] text-muted-foreground">
                    {dmgLabel}
                    {enemy.applyWay ? ` · ${APPLY_WAY_DISPLAY[enemy.applyWay]}` : ""}
                </span>
            </div>

            <div className="text-right">
                <span className="font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">{enemy.flatStats.maxHp.toLocaleString()}</span>
                <div className="mt-1 h-0.5 overflow-hidden rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_14%,transparent)]">
                    <div className="h-full" style={{ width: `${hpPct}%`, background: tok.accent }} />
                </div>
            </div>
        </Link>
    );
}
