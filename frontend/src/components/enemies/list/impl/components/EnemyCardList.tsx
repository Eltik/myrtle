import { Link } from "@tanstack/react-router";
import { Fragment, useState } from "react";
import { assetURL } from "#/lib/assets";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { APPLY_WAY_LABEL_KEY, ENEMY_LEVEL_LABEL_KEY, LIST_GRID_COLS } from "../constants";
import type { messages as listConstantsMessages } from "../constants.messages";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";
import type { IEnemyStatMaxByLevel, IEnemyView } from "../types";
import { DamageDots, LevelBadge } from "./atoms";
import type { messages } from "./EnemyCardList.messages";
import { EnemyPlaceholder } from "./EnemyPlaceholder";

interface IEnemyCardListProps {
    enemy: IEnemyView;
    statMax: IEnemyStatMaxByLevel;
}

export function EnemyCardList({ enemy, statMax }: IEnemyCardListProps) {
    const t: TypedT<typeof messages> = useT("enemies");
    const tConst: TypedT<typeof listConstantsMessages> = useT("enemies");
    const f = useFormatters();
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait && enemy.portrait ? assetURL(enemy.portrait) : undefined;
    const tierMaxHp = statMax[enemy.enemyLevel].hp;
    const hpPct = tierMaxHp > 0 ? Math.min(100, (enemy.flatStats.maxHp / tierMaxHp) * 100) : 0;

    return (
        <Link
            to="/enemies/$id"
            params={{ id: enemy.enemyId }}
            className="enemy-row group relative grid w-full cursor-pointer items-center gap-3.5 rounded-lg border border-transparent bg-card/50 px-3.5 py-2.5 text-left transition-colors before:absolute before:top-1/2 before:left-0 before:h-8 before:w-0.5 before:-translate-y-1/2 before:rounded-full before:bg-(--level-stripe) before:opacity-60 before:transition-opacity hover:border-border hover:bg-card hover:before:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring max-[900px]:grid-cols-[48px_1fr_auto]! max-[900px]:[&>:nth-child(4)]:hidden max-[900px]:[&>:nth-child(5)]:hidden"
            // The level stripe is a ::before, not a child, so the nth-child column hiding below 900px still counts only the cells.
            style={{ gridTemplateColumns: LIST_GRID_COLS, ["--level-stripe" as string]: enemy.enemyLevel === "NORMAL" ? "transparent" : tok.accent }}
            aria-label={t("cardList.aria", { name: enemy.name, index: enemy.enemyIndex })}
        >
            <div className="relative h-12 w-12 overflow-hidden rounded-md border border-[color-mix(in_oklch,var(--border)_60%,transparent)] bg-[color-mix(in_oklch,var(--muted)_40%,transparent)]">
                {hasPortrait && portraitSrc ? (
                    <img src={portraitSrc} alt={t("cardList.portraitAlt", { name: enemy.name })} loading="lazy" decoding="async" onError={() => setImgError(true)} className="absolute inset-0 h-full w-full object-contain transition-transform duration-200 ease-out group-hover:scale-110" />
                ) : (
                    <EnemyPlaceholder className="absolute inset-0 h-full w-full p-1.5" />
                )}
            </div>

            <div className="flex min-w-0 flex-col gap-0.75">
                <span className="truncate font-sans font-semibold text-[13px] text-foreground leading-[1.1]">{enemy.name}</span>
                <span className="font-medium font-mono text-[10px] text-muted-foreground leading-none">
                    {enemy.enemyIndex}
                    {enemy.race ? ` · ${enemy.race}` : ""}
                </span>
            </div>

            <div className="justify-self-start">{enemy.enemyLevel === "NORMAL" ? <span className="font-medium font-sans text-[11.5px] text-muted-foreground">{tConst(ENEMY_LEVEL_LABEL_KEY.NORMAL)}</span> : <LevelBadge level={enemy.enemyLevel} />}</div>

            <div className="inline-flex items-center gap-2">
                <DamageDots types={enemy.damageType} size={7} decorative />
                {/* Each damage type in its ink: 4.89 to 5.39 on the row's card/50 over the page, light; 5.69+ dark. */}
                <span className="truncate font-sans text-[12.5px] text-muted-foreground">
                    {enemy.damageType.length
                        ? enemy.damageType.map((dt, i) => (
                              <Fragment key={dt}>
                                  {i > 0 ? " · " : ""}
                                  <span style={{ color: DAMAGE_TOKENS[dt]?.ink }}>{DAMAGE_TOKENS[dt] ? tConst(DAMAGE_TOKENS[dt].labelKey) : dt}</span>
                              </Fragment>
                          ))
                        : "-"}
                    {enemy.applyWay ? ` · ${tConst(APPLY_WAY_LABEL_KEY[enemy.applyWay])}` : ""}
                </span>
            </div>

            <div className="text-right">
                <span className="font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">{f.number(enemy.flatStats.maxHp)}</span>
                <div className="mt-1 h-0.5 overflow-hidden rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_14%,transparent)]">
                    <div className="h-full" style={{ width: `${hpPct}%`, background: tok.accent }} />
                </div>
            </div>
        </Link>
    );
}
