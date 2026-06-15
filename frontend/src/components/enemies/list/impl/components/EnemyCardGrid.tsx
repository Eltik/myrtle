import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { env } from "#/env";
import { cn } from "#/lib/utils";
import { LEVEL_TOKENS } from "../tokens";
import type { IEnemyStatMaxByLevel, IEnemyView } from "../types";
import { DamageDots, LevelBadge, StatBar } from "./atoms";
import styles from "./EnemyCardGrid.module.css";
import { EnemyPlaceholder } from "./EnemyPlaceholder";

interface IEnemyCardGridProps {
    enemy: IEnemyView;
    statMax: IEnemyStatMaxByLevel;
}

export function EnemyCardGrid({ enemy, statMax }: IEnemyCardGridProps) {
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait ? `${env.VITE_BACKEND_URL ?? ""}/api/assets${enemy.portrait}` : undefined;
    const tierMax = statMax[enemy.enemyLevel];

    return (
        <Link
            to="/enemies/$id"
            params={{ id: enemy.enemyId }}
            className={cn(
                "enemy-card group relative flex w-full cursor-pointer flex-col overflow-hidden rounded-md border bg-card p-0 text-left hover:rounded-lg hover:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                styles["card-hover-transition"],
            )}
            style={{
                borderColor: enemy.enemyLevel === "NORMAL" ? "color-mix(in oklch, var(--border) 80%, transparent)" : tok.accentSoft,
            }}
            aria-label={`${enemy.name} (${enemy.enemyIndex})`}
        >
            <div className="relative aspect-square w-full bg-[color-mix(in_oklch,var(--muted)_50%,transparent)]">
                <div className="absolute inset-0 origin-center transition-transform duration-150 ease-in-out group-hover:scale-105 motion-reduce:transition-none motion-reduce:group-hover:scale-100">
                    {hasPortrait && portraitSrc ? <img src={portraitSrc} alt={`${enemy.name} portrait`} loading="lazy" decoding="async" onError={() => setImgError(true)} className="block h-full w-full object-contain" /> : <EnemyPlaceholder className="h-full w-full p-4.5" />}
                </div>

                <span className="absolute top-1.5 left-1.5 rounded-sm bg-background/75 px-1 py-0.5 font-medium font-mono text-[8.5px] text-muted-foreground uppercase leading-none tracking-[0.12em] backdrop-blur-[6px] sm:top-2 sm:left-2 sm:px-1.25 sm:py-0.75 sm:text-[9px] sm:tracking-[0.14em]">{enemy.enemyIndex}</span>

                <span className="absolute top-1.5 right-1.5 sm:top-2 sm:right-2">
                    <LevelBadge level={enemy.enemyLevel} size="sm" />
                </span>

                {enemy.enemyLevel !== "NORMAL" && <span aria-hidden="true" className="absolute right-0 bottom-0 left-0 h-0.5" style={{ background: tok.accent }} />}
            </div>

            <div className="flex flex-col gap-2 px-2 pt-2 pb-2.5 sm:px-3 sm:pt-2.5 sm:pb-3">
                <div className="flex min-h-6 items-start gap-1.5 sm:min-h-7.5">
                    <h3 className="m-0 line-clamp-2 min-w-0 flex-1 font-sans font-semibold text-[11px] text-foreground uppercase leading-[1.15] tracking-[0.02em] sm:text-[12.5px]">{enemy.name}</h3>
                    <DamageDots types={enemy.damageType} />
                </div>

                <div className="flex flex-col gap-1">
                    <StatBar label="HP" value={enemy.flatStats.maxHp} max={tierMax.hp} color={tok.accent} />
                    <StatBar label="ATK" value={enemy.flatStats.atk} max={tierMax.atk} color="color-mix(in oklch, var(--foreground) 50%, transparent)" />
                    <StatBar label="DEF" value={enemy.flatStats.def} max={tierMax.def} color="color-mix(in oklch, var(--foreground) 30%, transparent)" />
                </div>
            </div>
        </Link>
    );
}
