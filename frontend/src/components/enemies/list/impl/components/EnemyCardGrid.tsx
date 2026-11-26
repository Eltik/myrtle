import { Link } from "@tanstack/react-router";
import { useState } from "react";
import { assetURL } from "#/lib/assets";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { LEVEL_TOKENS } from "../tokens";
import type { IEnemyStatMaxByLevel, IEnemyView } from "../types";
import { DamageDots, LevelBadge, StatBar } from "./atoms";
import type { messages } from "./EnemyCardGrid.messages";
import styles from "./EnemyCardGrid.module.css";
import { EnemyPlaceholder } from "./EnemyPlaceholder";

interface IEnemyCardGridProps {
    enemy: IEnemyView;
    statMax: IEnemyStatMaxByLevel;
}

export function EnemyCardGrid({ enemy, statMax }: IEnemyCardGridProps) {
    const t: TypedT<typeof messages> = useT("enemies");
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait && enemy.portrait ? assetURL(enemy.portrait) : undefined;
    const tierMax = statMax[enemy.enemyLevel];

    return (
        <Link
            to="/enemies/$id"
            params={{ id: enemy.enemyId }}
            className={cn(
                "enemy-card group relative flex w-full cursor-pointer flex-col overflow-hidden rounded-2xl border bg-card p-0 text-left hover:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                styles["card-hover-transition"],
            )}
            style={{
                borderColor: enemy.enemyLevel === "NORMAL" ? "color-mix(in oklch, var(--border) 80%, transparent)" : tok.accentSoft,
            }}
        >
            <div className="relative aspect-square w-full bg-[color-mix(in_oklch,var(--muted)_50%,transparent)]">
                <div className="absolute inset-0 origin-center transition-transform duration-150 ease-in-out group-hover:scale-105 motion-reduce:transition-none motion-reduce:group-hover:scale-100">
                    {hasPortrait && portraitSrc ? <img src={portraitSrc} alt="" loading="lazy" decoding="async" onError={() => setImgError(true)} className="block h-full w-full object-contain" /> : <EnemyPlaceholder className="h-full w-full p-4.5" />}
                </div>

                <span className="absolute top-1.5 left-1.5 rounded-sm bg-background/95 px-1 py-0.5 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:top-2 sm:left-2 sm:px-1.25 sm:py-0.75 sm:tracking-[0.14em]">{enemy.enemyIndex}</span>

                {/* The badge's 15% tint was see-through: the dark ink measured 2.23:1 over white art.
                    On a solid background plate, whatever the art: Elite 4.56 / 6.58, Boss 4.54 / 7.11 (light / dark). */}
                <span className="absolute top-1.5 right-1.5 rounded-sm bg-background sm:top-2 sm:right-2">
                    <LevelBadge level={enemy.enemyLevel} size="sm" />
                </span>

                {enemy.enemyLevel !== "NORMAL" && <span aria-hidden="true" className="absolute right-0 bottom-0 left-0 h-0.5" style={{ background: tok.accent }} />}
            </div>

            <div className="flex flex-col gap-2 px-2 pt-2 pb-2.5 sm:px-3 sm:pt-2.5 sm:pb-3">
                {/* Two reserved lines, so a one-line name keeps its HP/ATK/DEF rows level with a two-line neighbour's. */}
                <div className="flex items-start gap-1.5">
                    <h3 className="m-0 line-clamp-2 min-h-[2lh] min-w-0 flex-1 font-sans font-semibold text-[11px] text-foreground leading-[1.15] sm:text-[12.5px]">{enemy.name}</h3>
                    <DamageDots types={enemy.damageType} size={8} />
                </div>

                {/* Every bar in the level's hue (Elite amber, Boss red, Normal grey); the label and the value beside each bar say which stat. */}
                <div className="flex flex-col gap-1">
                    <StatBar label={t("cardGrid.hp")} value={enemy.flatStats.maxHp} max={tierMax.hp} color={tok.ink} />
                    <StatBar label={t("cardGrid.atk")} value={enemy.flatStats.atk} max={tierMax.atk} color={tok.ink} />
                    <StatBar label={t("cardGrid.def")} value={enemy.flatStats.def} max={tierMax.def} color={tok.ink} />
                </div>
            </div>
        </Link>
    );
}
